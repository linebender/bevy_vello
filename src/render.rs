use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::ExtractedCamera,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::{RenderAsset, RenderAssetPlugin, RenderAssets},
        renderer::{RenderDevice, RenderQueue},
        view::ExtractedView,
        RenderApp, RenderSet,
    },
};
use vello::{
    kurbo::Affine, RenderParams, Renderer, RendererOptions, Scene, SceneBuilder, SceneFragment,
};

use crate::{
    assets::vector::{Vector, VelloVector},
    font::VelloFont,
    ColorPaletteSwap, Layer, VelloText,
};

#[derive(Resource)]
struct VelloRenderer(Renderer);

impl FromWorld for VelloRenderer {
    fn from_world(world: &mut World) -> Self {
        let device = world.get_resource::<RenderDevice>().unwrap();
        VelloRenderer(
            Renderer::new(
                device.wgpu_device(),
                &RendererOptions {
                    surface_format: None,
                },
            )
            .expect("no gpu device"),
        )
    }
}

pub struct VelloRenderPlugin;

#[derive(Resource)]
pub struct VelatoRenderer(pub velato::Renderer);

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        render_app.init_resource::<VelloRenderer>();
        render_app.insert_resource(VelatoRenderer(velato::Renderer::new()));

        render_app.add_system(prepare_vector_affines.in_set(RenderSet::Prepare));
        render_app.add_system(prepare_vector_composition_edits.in_set(RenderSet::Prepare));
        render_app.add_system(prepare_text_affines.in_set(RenderSet::Prepare));
        render_app.add_system(render_scene.in_set(RenderSet::Render));

        app.add_plugin(ExtractComponentPlugin::<ExtractedRenderVector>::default())
            .add_plugin(ExtractComponentPlugin::<ExtractedRenderText>::default())
            .add_plugin(ExtractComponentPlugin::<SSRenderTarget>::default())
            .add_plugin(RenderAssetPlugin::<VelloVector>::default())
            .add_plugin(RenderAssetPlugin::<VelloFont>::default())
            .add_system(tag_vectors_for_render);
    }
}

fn prepare_vector_composition_edits(
    mut render_vectors: Query<&mut ExtractedRenderVector>,
    mut render_vector_assets: ResMut<RenderAssets<VelloVector>>,
) {
    // Big-O: O(n), where n = shapes;
    // Nesting: "vectors * layers * shape groups * shapes"
    'vectors: for render_vector in render_vectors.iter_mut() {
        let Some(vector) = render_vector_assets.get_mut(&render_vector.vector) else {
            continue 'vectors;
        };
        let Vector::Animated(ref mut composition) = vector.data else {
            continue 'vectors;
        };
        'layers: for (_layer_index, layer) in composition.layers.iter_mut().enumerate() {
            let velato::model::Content::Shape(ref mut shapes) = layer.content else {
                continue 'layers;
            };
            'shapegroups: for (shape_index, shape) in shapes.iter_mut().enumerate() {
                let velato::model::Shape::Group(ref mut shapes, _transform) = shape else {
                    continue 'shapegroups;
                };
                'shapes: for shape in shapes.iter_mut() {
                    let velato::model::Shape::Draw(ref mut draw) = shape else {
                        continue 'shapes;
                    };
                    let velato::model::Brush::Fixed(ref mut brush) = draw.brush else {
                        continue 'shapes;
                    };
                    let vello::peniko::Brush::Solid(ref mut solid) = brush else {
                        continue 'shapes;
                    };
                    // println!(
                    //     "layer '{}', shape {}: color {:?}",
                    //     layer.name,
                    //     shape_index,
                    //     (solid.r, solid.g, solid.b, solid.a)
                    // );

                    if let Some(ColorPaletteSwap { colors }) = &render_vector.color_pallette_swap {
                        for ((layer_name, shape_indices), color) in colors.iter() {
                            if layer.name.contains(layer_name)
                                && shape_indices.contains(&shape_index)
                            {
                                *solid = vello::peniko::Color::rgba(
                                    color.r().into(),
                                    color.g().into(),
                                    color.b().into(),
                                    color.a().into(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn prepare_vector_affines(
    camera: Query<(&ExtractedCamera, &ExtractedView)>,
    mut render_vectors: Query<&mut ExtractedRenderVector>,
    render_vector_assets: Res<RenderAssets<VelloVector>>,
) {
    let (camera, view) = camera.single();
    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
    let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
    for mut render_vector in render_vectors.iter_mut() {
        let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
            [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
            [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .transpose();

        let world_transform = render_vector.transform;
        let local_matrix = match render_vector_assets.get(&render_vector.vector) {
            Some(render_instance_data) => render_instance_data.local_matrix,
            None => Mat4::default(),
        };
        let mut model_matrix = world_transform.compute_matrix() * local_matrix;
        model_matrix.w_axis.y *= -1.0;

        let (projection_mat, view_mat) = {
            let mut view_mat = view.transform.compute_matrix();
            view_mat.w_axis.y *= -1.0;

            (view.projection, view_mat)
        };

        let view_proj_matrix = projection_mat * view_mat.inverse();

        let raw_transform = ndc_to_pixels_matrix * view_proj_matrix * model_matrix;

        let transform: [f32; 16] = raw_transform.to_cols_array();

        // | a c e |
        // | b d f |
        // | 0 0 1 |
        let transform: [f64; 6] = [
            transform[0] as f64,  // a
            -transform[1] as f64, // b
            -transform[4] as f64, // c
            transform[5] as f64,  // d
            transform[12] as f64, // e
            transform[13] as f64, // f
        ];

        let affine = Affine::new(transform);
        render_vector.affine = affine;
    }
}

fn prepare_text_affines(
    camera: Query<(&ExtractedCamera, &ExtractedView)>,
    mut render_texts: Query<&mut ExtractedRenderText>,
) {
    let (camera, view) = camera.single();
    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
    let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
    for mut render_text in render_texts.iter_mut() {
        let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
            [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
            [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .transpose();

        let world_transform = render_text.transform;

        let mut model_matrix = world_transform.compute_matrix();
        model_matrix.w_axis.y *= -1.0;

        let (projection_mat, view_mat) = {
            let mut view_mat = view.transform.compute_matrix();
            view_mat.w_axis.y *= -1.0;

            (view.projection, view_mat)
        };

        let view_proj_matrix = projection_mat * view_mat.inverse();
        let vello_matrix = ndc_to_pixels_matrix * view_proj_matrix;

        let raw_transform = vello_matrix * model_matrix;

        let transform: [f32; 16] = raw_transform.to_cols_array();

        // | a c e |
        // | b d f |
        // | 0 0 1 |
        let transform: [f64; 6] = [
            transform[0] as f64,  // a
            -transform[1] as f64, // b
            -transform[4] as f64, // c
            transform[5] as f64,  // d
            transform[12] as f64, // e
            transform[13] as f64, // f
        ];

        let affine = Affine::new(transform);
        render_text.affine = affine;
        render_text.vello_matrix = vello_matrix;
    }
}

/// Transforms all the vectors extracted from the game world and places them in
/// a scene, and renders the scene to a texture with WGPU
#[allow(clippy::complexity)]
fn render_scene(
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

        // Apply transforms to the respective fragments and add them to the
        // scene to be rendered
        for ExtractedRenderVector { vector, affine, .. } in vector_render_queue.iter() {
            match vector_render_assets.get(vector) {
                Some(RenderInstanceData {
                    data: Vector::Static(fragment),
                    ..
                }) => {
                    builder.append(fragment, Some(*affine));
                }
                Some(RenderInstanceData {
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

#[derive(Component, Default)]
pub struct SSRenderTarget(pub Handle<Image>);

impl ExtractComponent for SSRenderTarget {
    type Query = &'static SSRenderTarget;

    type Filter = ();

    type Out = Self;

    fn extract_component(
        ss_render_target: bevy::ecs::query::QueryItem<'_, Self::Query>,
    ) -> Option<Self> {
        Some(Self(ss_render_target.0.clone()))
    }
}

#[derive(Component, Clone)]
struct ExtractedRenderVector {
    vector: Handle<VelloVector>,
    transform: GlobalTransform,
    affine: Affine,
    layer: Layer,
    color_pallette_swap: Option<ColorPaletteSwap>,
}

impl ExtractComponent for ExtractedRenderVector {
    type Query = (
        &'static Handle<VelloVector>,
        &'static Layer,
        &'static GlobalTransform,
        Option<&'static ColorPaletteSwap>,
    );

    type Filter = &'static RenderReadyTag;
    type Out = Self;

    fn extract_component(
        (vello_vector_handle, layer, transform, color_pallette_swap): bevy::ecs::query::QueryItem<
            '_,
            Self::Query,
        >,
    ) -> Option<Self> {
        Some(Self {
            vector: vello_vector_handle.clone(),
            transform: *transform,
            affine: Affine::default(),
            layer: *layer,
            color_pallette_swap: color_pallette_swap.cloned(),
        })
    }
}

#[derive(Component, Clone)]
struct ExtractedRenderText {
    font: Handle<VelloFont>,
    text: VelloText,
    transform: GlobalTransform,
    affine: Affine,
    vello_matrix: Mat4,
}

impl ExtractComponent for ExtractedRenderText {
    type Query = (
        &'static Handle<VelloFont>,
        &'static VelloText,
        &'static GlobalTransform,
    );

    type Filter = ();
    type Out = Self;

    fn extract_component(
        (vello_font_handle, text, transform): bevy::ecs::query::QueryItem<'_, Self::Query>,
    ) -> Option<Self> {
        Some(Self {
            font: vello_font_handle.clone(),
            text: text.clone(),
            transform: *transform,
            affine: Affine::default(),
            vello_matrix: Mat4::default(),
        })
    }
}

#[derive(Component)]
pub struct RenderReadyTag;

fn tag_vectors_for_render(
    mut commands: Commands,
    vector_assets: ResMut<Assets<VelloVector>>,
    vectors: Query<(Entity, &Handle<VelloVector>), Without<RenderReadyTag>>,
) {
    for (entity, handle) in vectors.iter() {
        if vector_assets.get(handle).is_some() {
            commands.entity(entity).insert(RenderReadyTag);
        }
    }
}

#[derive(Clone)]
pub struct VectorAssetData {
    vector: Vector,
    local_transform: Transform,
}

impl RenderAsset for VelloVector {
    type ExtractedAsset = VectorAssetData;

    type PreparedAsset = RenderInstanceData;

    type Param = ();

    fn extract_asset(&self) -> Self::ExtractedAsset {
        VectorAssetData {
            vector: self.data.clone(),
            local_transform: self.local_transform,
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
pub struct RenderInstanceData {
    pub local_matrix: Mat4,
    pub data: Vector,
}

impl From<VectorAssetData> for RenderInstanceData {
    fn from(value: VectorAssetData) -> Self {
        let local_matrix = value.local_transform.compute_matrix().inverse();
        let vector_data = value.vector;

        RenderInstanceData {
            data: vector_data,
            local_matrix,
        }
    }
}

impl Default for RenderInstanceData {
    fn default() -> Self {
        Self {
            data: Vector::Static(Arc::new(SceneFragment::default())),
            local_matrix: Mat4::default(),
        }
    }
}
