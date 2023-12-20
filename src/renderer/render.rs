use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{RenderAsset, RenderAssets},
        renderer::{RenderDevice, RenderQueue},
    },
};
use vello::{RenderParams, Scene, SceneBuilder};

use crate::{
    assets::vector::{Vector, VelloVector},
    font::VelloFont,
    Layer,
};

use super::{
    extract::{ExtractedRenderText, ExtractedRenderVector},
    prepare::PreparedAffine,
    LottieRenderer, SSRenderTarget, VelloRenderer,
};

#[derive(Clone)]
pub struct ExtractedVectorAssetData {
    local_transform_bottom_center: Transform,
    local_transform_center: Transform,
    size: Vec2,
}

impl RenderAsset for VelloVector {
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
    renderer: Option<NonSendMut<VelloRenderer>>,
    ss_render_target: Query<&SSRenderTarget>,
    render_vectors: Query<(&PreparedAffine, &ExtractedRenderVector)>,
    query_render_texts: Query<(&PreparedAffine, &ExtractedRenderText)>,
    mut font_render_assets: ResMut<RenderAssets<VelloFont>>,
    gpu_images: Res<RenderAssets<Image>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut velato_renderer: ResMut<LottieRenderer>,
    time: Res<Time>,
) {
    let mut renderer = if let Some(renderer) = renderer {
        renderer
    } else {
        return;
    };

    if let Ok(SSRenderTarget(render_target_image)) = ss_render_target.get_single() {
        let gpu_image = gpu_images.get(render_target_image).unwrap();
        let mut scene = Scene::default();
        let mut builder = SceneBuilder::for_scene(&mut scene);

        // Background items: z ordered
        let mut vector_render_queue: Vec<(_, _)> = render_vectors
            .iter()
            .filter(|(_, v)| v.layer == Layer::Background)
            .collect();
        vector_render_queue.sort_by(|(_, a), (_, b)| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Shadow items: z ordered
        let mut shadow_items: Vec<(_, _)> = render_vectors
            .iter()
            .filter(|(_, v)| v.layer == Layer::Shadow)
            .collect();
        shadow_items.sort_by(|(_, a), (_, b)| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut shadow_items);

        // Ground items: y ordered
        let mut middle_items: Vec<(_, _)> = render_vectors
            .iter()
            .filter(|(_, v)| v.layer == Layer::Ground)
            .collect();
        middle_items.sort_by(|(_, a), (_, b)| {
            let a = a.transform.translation().y;
            let b = b.transform.translation().y;
            b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut middle_items);

        // Foreground items: z ordered
        let mut fg_items: Vec<(_, _)> = render_vectors
            .iter()
            .filter(|(_, v)| v.layer == Layer::Foreground)
            .collect();
        fg_items.sort_by(|(_, a), (_, b)| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut fg_items);

        // UI items
        let mut ui_items: Vec<(_, _)> = render_vectors
            .iter()
            .filter(|(_, v)| v.layer == Layer::UI)
            .collect();
        ui_items.sort_by(|(_, a), (_, b)| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut ui_items);

        // Apply transforms to the respective fragments and add them to the
        // scene to be rendered
        for (
            PreparedAffine(affine),
            ExtractedRenderVector {
                render_data: vector_data,
                ..
            },
        ) in vector_render_queue.iter()
        {
            match &vector_data.data {
                Vector::Static(fragment) => {
                    builder.append(fragment, Some(*affine));
                }
                Vector::Animated(composition) => {
                    velato_renderer.0.render(
                        composition,
                        time.elapsed_seconds(),
                        *affine,
                        1.0,
                        &mut builder,
                    );
                }
            }
        }

        for (&PreparedAffine(affine), ExtractedRenderText { font, text, .. }) in
            query_render_texts.iter()
        {
            if let Some(font) = font_render_assets.get_mut(font) {
                font.render_centered(&mut builder, text.size, affine, &text.content);
            }
        }

        if !vector_render_queue.is_empty() {
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
