use super::{
    extract::{ExtractedRenderText, ExtractedRenderVector},
    prepare::PreparedAffine,
    BevyVelloRenderer, LottieRenderer, SSRenderTarget,
};
use crate::{CoordinateSpace, VectorFile, VelloFont};
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        renderer::{RenderDevice, RenderQueue},
    },
};
use vello::{RenderParams, Scene, SceneBuilder};

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
    vello_renderer: Option<NonSendMut<BevyVelloRenderer>>,
    mut velottie_renderer: ResMut<LottieRenderer>,
) {
    let mut renderer = if let Some(renderer) = vello_renderer {
        renderer
    } else {
        return;
    };

    if let Ok(SSRenderTarget(render_target_image)) =
        ss_render_target.get_single()
    {
        let gpu_image = gpu_images.get(render_target_image).unwrap();
        let mut scene = Scene::default();
        let mut builder = SceneBuilder::for_scene(&mut scene);

        enum RenderItem<'a> {
            Vector(&'a ExtractedRenderVector),
            Text(&'a ExtractedRenderText),
        }
        let mut render_queue: Vec<(
            f32,
            CoordinateSpace,
            (&PreparedAffine, RenderItem),
        )> = render_vectors
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
        for (_, _, (&PreparedAffine(affine), render_item)) in
            render_queue.iter_mut()
        {
            match render_item {
                RenderItem::Vector(ExtractedRenderVector {
                    asset,
                    theme,
                    alpha,
                    playhead,
                    ..
                }) => match &asset.data {
                    VectorFile::Svg {
                        original: fragment, ..
                    } => {
                        builder.append(fragment, Some(affine));
                    }
                    VectorFile::Lottie { composition } => {
                        debug!("playhead: {playhead}");
                        velottie_renderer.0.render(
                            {
                                theme
                                    .as_ref()
                                    .map(|cs| cs.recolor(composition))
                                    .as_ref()
                                    .unwrap_or(composition)
                            },
                            *playhead,
                            affine,
                            *alpha,
                            &mut builder,
                        );
                    }
                },
                RenderItem::Text(ExtractedRenderText {
                    font, text, ..
                }) => {
                    if let Some(font) = font_render_assets.get_mut(font) {
                        font.render(&mut builder, affine, text);
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
                        base_color: vello::peniko::Color::BLACK
                            .with_alpha_factor(0.0),
                        width: gpu_image.size.x as u32,
                        height: gpu_image.size.y as u32,
                        antialiasing_method: vello::AaConfig::Area,
                    },
                )
                .unwrap();
        }
    }
}
