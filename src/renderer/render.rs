use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{RenderAsset, RenderAssets},
        renderer::{RenderDevice, RenderQueue},
    },
};
use vello::{RenderParams, Scene, SceneBuilder, SceneFragment};

use crate::{
    assets::vector::{Vector, VelloVector},
    font::VelloFont,
    Layer,
};

use super::{
    extract::{ExtractedRenderText, ExtractedRenderVector},
    SSRenderTarget, VelatoRenderer, VelloRenderer,
};

#[derive(Clone)]
pub struct ExtractedVectorAssetData {
    vector: Vector,
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
            vector: self.data.clone(),
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
    pub data: Vector,
    pub size: Vec2,
}

impl From<ExtractedVectorAssetData> for PreparedVectorAssetData {
    fn from(value: ExtractedVectorAssetData) -> Self {
        let local_bottom_center_matrix = value
            .local_transform_bottom_center
            .compute_matrix()
            .inverse();
        let local_center_matrix = value.local_transform_center.compute_matrix().inverse();
        let vector_data = value.vector;
        let size = value.size;

        PreparedVectorAssetData {
            data: vector_data,
            local_bottom_center_matrix,
            local_center_matrix,
            size,
        }
    }
}

impl Default for PreparedVectorAssetData {
    fn default() -> Self {
        Self {
            data: Vector::Static(Arc::new(SceneFragment::default())),
            local_bottom_center_matrix: Mat4::default(),
            local_center_matrix: Mat4::default(),
            size: Vec2::default(),
        }
    }
}

/// Transforms all the vectors extracted from the game world and places them in
/// a scene, and renders the scene to a texture with WGPU
#[allow(clippy::complexity)]
pub fn render_scene(
    mut renderer: ResMut<VelloRenderer>,
    ss_render_target: Query<&SSRenderTarget>,
    render_vectors: Query<&ExtractedRenderVector>,
    query_render_texts: Query<&ExtractedRenderText>,
    vector_render_assets: Res<RenderAssets<VelloVector>>,
    mut font_render_assets: ResMut<RenderAssets<VelloFont>>,
    gpu_images: Res<RenderAssets<Image>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut velato_renderer: ResMut<VelatoRenderer>,
    time: Res<Time>,
) {
    if let Ok(SSRenderTarget(render_target_image)) = ss_render_target.get_single() {
        let gpu_image = gpu_images.get(render_target_image).unwrap();
        let mut scene = Scene::default();
        let mut builder = SceneBuilder::for_scene(&mut scene);

        // Background items: z ordered
        let mut vector_render_queue: Vec<ExtractedRenderVector> = render_vectors
            .iter()
            .filter(|v| v.layer == Layer::Background)
            .cloned()
            .collect();
        vector_render_queue.sort_by(|a, b| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Shadow items: z ordered
        let mut shadow_items: Vec<ExtractedRenderVector> = render_vectors
            .iter()
            .filter(|v| v.layer == Layer::Shadow)
            .cloned()
            .collect();
        shadow_items.sort_by(|a, b| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut shadow_items);

        // Middle items: y ordered
        let mut middle_items: Vec<ExtractedRenderVector> = render_vectors
            .iter()
            .filter(|v| v.layer == Layer::Middle)
            .cloned()
            .collect();
        middle_items.sort_by(|a, b| {
            let a = a.transform.translation().y;
            let b = b.transform.translation().y;
            b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut middle_items);

        // Foreground items: z ordered
        let mut fg_items: Vec<ExtractedRenderVector> = render_vectors
            .iter()
            .filter(|v| v.layer == Layer::Foreground)
            .cloned()
            .collect();
        fg_items.sort_by(|a, b| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut fg_items);

        // Foreground items:
        let mut ui_items: Vec<ExtractedRenderVector> = render_vectors
            .iter()
            .filter(|v| v.layer == Layer::UI)
            .cloned()
            .collect();
        ui_items.sort_by(|a, b| {
            let a = a.transform.translation().z;
            let b = b.transform.translation().z;
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_render_queue.append(&mut ui_items);

        // Apply transforms to the respective fragments and add them to the
        // scene to be rendered
        for ExtractedRenderVector { vector, affine, .. } in vector_render_queue.iter() {
            match vector_render_assets.get(vector) {
                Some(PreparedVectorAssetData {
                    data: Vector::Static(fragment),
                    ..
                }) => {
                    builder.append(fragment, Some(*affine));
                }
                Some(PreparedVectorAssetData {
                    data: Vector::Animated(composition),
                    ..
                }) => {
                    velato_renderer.0.render(
                        composition,
                        time.elapsed_seconds(),
                        *affine,
                        1.0,
                        &mut builder,
                    );
                }
                None => {}
            }
        }

        for ExtractedRenderText {
            font, text, affine, ..
        } in query_render_texts.iter()
        {
            if let Some(font) = font_render_assets.get_mut(&font) {
                font.render_centered(&mut builder, text.size, *affine, &text.content);
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
                    },
                )
                .unwrap();
        }
    }
}
