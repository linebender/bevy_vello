use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::camera::ExtractedCamera;
use bevy::render::sync_world::TemporaryRenderEntity;
use bevy::render::view::ExtractedView;
use vello::kurbo::Affine;

use crate::integrations::scene::{UiVelloScene, VelloScene2d};
use crate::render::prepare::PreparedAffine;
use crate::render::{SkipEncoding, VelloEntityCountData, VelloView};

#[derive(Component, Clone)]
pub struct ExtractedVelloScene2d {
    pub scene: VelloScene2d,
    pub transform: GlobalTransform,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloScene {
    pub scene: UiVelloScene,
    pub ui_transform: UiGlobalTransform,
    pub ui_node: ComputedNode,
    pub ui_render_target: ComputedUiRenderTargetInfo,
}

pub fn extract_world_scenes(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &VelloScene2d,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&RenderLayers>,
            ),
            (Without<SkipEncoding>, Without<Node>),
        >,
    >,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_scenes = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (scene, transform, view_visibility, inherited_visibility, render_layers) in
        query_scenes.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloScene2d {
                    transform: *transform,
                    scene: scene.clone(),
                })
                .insert(TemporaryRenderEntity);
            n_scenes += 1;
        }
    }

    frame_data.n_world_scenes = n_scenes;
}

pub fn extract_ui_scenes(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &UiVelloScene,
                &ComputedNode,
                &ComputedUiRenderTargetInfo,
                &UiGlobalTransform,
                &InheritedVisibility,
                Option<&RenderLayers>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_scenes = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (scene, ui_node, ui_render_target, ui_transform, inherited_visibility, render_layers) in
        query_scenes.iter()
    {
        // Skip if visibility conditions are not met.
        // UI does not check view visibility, only inherited visibility.
        if !inherited_visibility.get() {
            continue;
        }
        // Check if any camera renders this asset
        let asset_render_layers = render_layers.unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedUiVelloScene {
                    scene: scene.clone(),
                    ui_transform: *ui_transform,
                    ui_node: *ui_node,
                    ui_render_target: *ui_render_target,
                })
                .insert(TemporaryRenderEntity);
            n_scenes += 1;
        }
    }

    frame_data.n_ui_scenes = n_scenes;
}

pub fn prepare_scene_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView), (With<Camera2d>, With<VelloView>)>,
    render_entities: Query<(Entity, &ExtractedVelloScene2d)>,
    render_ui_entities: Query<(Entity, &ExtractedUiVelloScene)>,
) {
    for (camera, view) in views.iter() {
        // Render UI
        for (entity, render_entity) in render_ui_entities.iter() {
            let pixel_scale = render_entity.ui_render_target.scale_factor();
            let pixel_scale_matrix = Mat4::from_scale(Vec3::new(pixel_scale, pixel_scale, 1.0));
            let ui_transform = render_entity.ui_transform;
            let ui_node = render_entity.ui_node;

            // A transposed (flipped over its diagonal) PostScript matrix
            // | a c e |
            // | b d f |
            // | 0 0 1 |
            //
            // Components
            // | scale_x skew_x translate_x |
            // | skew_y scale_y translate_y |
            // | skew_z skew_z scale_z |
            //
            // rotate (z)
            // | cos(θ) -sin(θ) translate_x |
            // | sin(θ) cos(θ) translate_y |
            // | skew_z skew_z scale_z |
            //
            // The order of operations is important, as it affects the final transformation matrix.
            //
            // Order of operations:
            // 1. Scale
            // 2. Rotate
            // 3. Translate
            let transform: [f64; 6] = {
                // Convert UiGlobalTransform to Mat4
                let mat2 = ui_transform.matrix2;
                let translation = ui_transform.translation;
                let model_matrix = Mat4::from_cols_array_2d(&[
                    [mat2.x_axis.x, mat2.x_axis.y, 0.0, 0.0],
                    [mat2.y_axis.x, mat2.y_axis.y, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [translation.x, translation.y, 0.0, 1.0],
                ]);

                // Apply node centering transformation
                // Get node center
                let local_center_matrix = {
                    let Vec2 {
                        x: width,
                        y: height,
                    } = ui_node.size();
                    Mat4::from_translation(Vec3::new(width / 2.0, height / 2.0, 0.0)).inverse()
                };

                // Transform chain: ui_transform (already in px) → pixel_scale
                let raw_transform = model_matrix * local_center_matrix * pixel_scale_matrix;
                let transform = raw_transform.to_cols_array();
                [
                    transform[0] as f64,  // a // scale_x
                    transform[1] as f64,  // b // skew_y
                    transform[4] as f64,  // c // skew_x
                    transform[5] as f64,  // d // scale_y
                    transform[12] as f64, // e // translate_x
                    transform[13] as f64, // f // translate_y
                ]
            };

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }

        // Render World
        for (entity, render_entity) in render_entities.iter() {
            let world_transform = render_entity.transform;

            // A transposed (flipped over its diagonal) PostScript matrix
            // | a c e |
            // | b d f |
            // | 0 0 1 |
            //
            // Components
            // | scale_x skew_x translate_x |
            // | skew_y scale_y translate_y |
            // | skew_z skew_z scale_z |
            //
            // rotate (z)
            // | cos(θ) -sin(θ) translate_x |
            // | sin(θ) cos(θ) translate_y |
            // | skew_z skew_z scale_z |
            //
            // The order of operations is important, as it affects the final transformation matrix.
            //
            // Order of operations:
            // 1. Scale
            // 2. Rotate
            // 3. Translate
            let transform: [f64; 6] = {
                let ndc_to_pixels_matrix = {
                    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
                    let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
                    Mat4::from_cols_array_2d(&[
                        [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                        [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ])
                    .transpose()
                };
                let view_proj_matrix = {
                    let mut view_mat = view.world_from_view.to_matrix();
                    // Flip Y-axis to match Vello's y-down coordinate space
                    view_mat.w_axis.y *= -1.0;
                    let proj_mat = view.clip_from_view;
                    proj_mat * view_mat.inverse()
                };
                let model_matrix = {
                    let mut model_matrix = world_transform.to_matrix();
                    // Flip Y-axis to match Vello's y-down coordinate space
                    model_matrix.w_axis.y *= -1.0;
                    model_matrix
                };

                // Transform chain: world → view → projection → NDC → pixels → pixel_scale
                let raw_transform = ndc_to_pixels_matrix * view_proj_matrix * model_matrix;
                let transform = raw_transform.to_cols_array();

                // Negate skew_x and skew_y to match rotation of the Bevy's y-up world
                [
                    transform[0] as f64,  // a // scale_x
                    -transform[1] as f64, // b // skew_y
                    -transform[4] as f64, // c // skew_x
                    transform[5] as f64,  // d // scale_y
                    transform[12] as f64, // e // translate_x
                    transform[13] as f64, // f // translate_y
                ]
            };

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}
