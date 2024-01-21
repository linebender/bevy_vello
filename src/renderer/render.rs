use super::{
    extract::{ExtractedRenderText, ExtractedRenderVector},
    prepare::PreparedAffine,
    LottieRenderer, SSRenderTarget, VelloRenderer,
};
use crate::{
    assets::vector::{Vector, VelloAsset},
    font::VelloFont,
    playback_settings::{PlaybackDirection, PlaybackSettings},
    CoordinateSpace,
};
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{RenderAsset, RenderAssets},
        renderer::{RenderDevice, RenderQueue},
    },
};
use vello::{RenderParams, Scene, SceneBuilder};
use vello_svg::usvg::strict_num::Ulps;

#[derive(Clone)]
pub struct ExtractedVectorAssetData {
    local_transform_bottom_center: Transform,
    local_transform_center: Transform,
    size: Vec2,
}

impl RenderAsset for VelloAsset {
    type ExtractedAsset = ExtractedVectorAssetData;

    type PreparedAsset = PreparedVectorAssetData;

    type Param = ();

    fn extract_asset(&self) -> Self::ExtractedAsset {
        ExtractedVectorAssetData {
            local_transform_bottom_center: self.local_transform_bottom_center,
            local_transform_center: self.local_transform_center,
            size: Vec2::new(self.width, self.height),
        }
    }

    fn prepare_asset(
        data: Self::ExtractedAsset,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        Ok(data.into())
    }
}

#[derive(TypeUuid, Clone)]
#[uuid = "39cadc56-aa9c-4543-3640-a018b74b5054"]
pub struct PreparedVectorAssetData {
    pub local_bottom_center_matrix: Mat4,
    pub local_center_matrix: Mat4,
    pub size: Vec2,
}

impl From<ExtractedVectorAssetData> for PreparedVectorAssetData {
    fn from(value: ExtractedVectorAssetData) -> Self {
        let local_bottom_center_matrix = value
            .local_transform_bottom_center
            .compute_matrix()
            .inverse();
        let local_center_matrix = value.local_transform_center.compute_matrix().inverse();
        let size = value.size;

        PreparedVectorAssetData {
            local_bottom_center_matrix,
            local_center_matrix,
            size,
        }
    }
}

/// Transforms all the vectors extracted from the game world and places them in
/// a scene, and renders the scene to a texture with WGPU
#[allow(clippy::complexity)]
pub fn render_scene(
    ss_render_target: Query<&SSRenderTarget>,
    render_vectors: Query<(&PreparedAffine, &ExtractedRenderVector)>,
    query_render_texts: Query<(&PreparedAffine, &ExtractedRenderText)>,
    mut font_render_assets: ResMut<RenderAssets<VelloFont>>,
    gpu_images: Res<RenderAssets<Image>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    vello_renderer: Option<NonSendMut<VelloRenderer>>,
    mut velottie_renderer: ResMut<LottieRenderer>,
    time: Res<Time>,
) {
    let mut renderer = if let Some(renderer) = vello_renderer {
        renderer
    } else {
        return;
    };

    if let Ok(SSRenderTarget(render_target_image)) = ss_render_target.get_single() {
        let gpu_image = gpu_images.get(render_target_image).unwrap();
        let mut scene = Scene::default();
        let mut builder = SceneBuilder::for_scene(&mut scene);

        enum RenderItem<'a> {
            Vector(&'a ExtractedRenderVector),
            Text(&'a ExtractedRenderText),
        }
        let mut render_queue: Vec<(f32, CoordinateSpace, (&PreparedAffine, RenderItem))> =
            render_vectors
                .iter()
                .map(|(a, b)| {
                    (
                        b.transform.translation().z,
                        b.render_mode,
                        (a, RenderItem::Vector(b)),
                    )
                })
                .collect();
        render_queue.extend(query_render_texts.iter().map(|(a, b)| {
            (
                b.transform.translation().z,
                b.render_mode,
                (a, RenderItem::Text(b)),
            )
        }));

        // Sort by render mode with screen space on top, then by z-index
        render_queue.sort_by(
            |(a_z_index, a_render_mode, _), (b_z_index, b_render_mode, _)| {
                let z_index = a_z_index
                    .partial_cmp(b_z_index)
                    .unwrap_or(std::cmp::Ordering::Equal);
                let render_mode = a_render_mode.cmp(b_render_mode);

                render_mode.then(z_index)
            },
        );

        // Apply transforms to the respective fragments and add them to the
        // scene to be rendered
        for (_, _, (&PreparedAffine(affine), render_item)) in render_queue.iter() {
            match render_item {
                RenderItem::Vector(ExtractedRenderVector {
                    render_data,
                    playback_settings,
                    ..
                }) => match &render_data.data {
                    Vector::Svg {
                        original: fragment, ..
                    } => {
                        builder.append(fragment, Some(affine));
                    }
                    Vector::Lottie {
                        original,
                        dirty,
                        playback_started,
                    } => {
                        let composition = dirty.as_ref().unwrap_or(original);
                        let t = if let Some(PlaybackSettings {
                            autoplay,
                            direction,
                            speed,
                            looping,
                            segments,
                        }) = playback_settings
                        {
                            let mut segments = composition.frames.start.max(segments.start)
                                ..composition.frames.end.min(segments.end);
                            if *direction == PlaybackDirection::Reverse {
                                segments = segments.end..segments.start;
                            }
                            if !autoplay {
                                // Static frame
                                match direction {
                                    PlaybackDirection::Normal => segments.start,
                                    PlaybackDirection::Reverse => segments.start.prev(),
                                }
                            } else {
                                let speed = speed * (*direction as i32 as f32);
                                let elapsed_t = playback_started.elapsed().as_secs_f32() * speed;
                                let anim_length =
                                    (segments.end - segments.start).abs() / composition.frame_rate;
                                if *looping {
                                    // Currently looping
                                    segments.start / composition.frame_rate
                                        + elapsed_t % anim_length
                                } else if elapsed_t.abs() >= anim_length {
                                    // Not looping - Playback finished, stay on the last frame
                                    match direction {
                                        PlaybackDirection::Normal => segments.end.prev(),
                                        PlaybackDirection::Reverse => segments.end.next(),
                                    }
                                } else {
                                    // Not looping - Still playing
                                    segments.start / composition.frame_rate + elapsed_t
                                }
                            }
                        } else {
                            time.elapsed_seconds()
                        };
                        velottie_renderer
                            .0
                            .render(composition, t, affine, 1.0, &mut builder);
                    }
                },
                RenderItem::Text(ExtractedRenderText { font, text, .. }) => {
                    if let Some(font) = font_render_assets.get_mut(font) {
                        font.render_centered(&mut builder, text.size, affine, &text.content);
                    }
                }
            }
        }

        if !render_queue.is_empty() {
            renderer
                .0
                .render_to_texture(
                    device.wgpu_device(),
                    &queue,
                    &scene,
                    &gpu_image.texture_view,
                    &RenderParams {
                        base_color: vello::peniko::Color::BLACK.with_alpha_factor(0.0),
                        width: gpu_image.size.x as u32,
                        height: gpu_image.size.y as u32,
                        antialiasing_method: vello::AaConfig::Area,
                    },
                )
                .unwrap();
        }
    }
}
