use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::camera::ExtractedCamera;
use bevy::render::sync_world::TemporaryRenderEntity;
use bevy::render::view::ExtractedView;
use vello::kurbo::Affine;

use crate::VelloRenderSpace;
use crate::prelude::VelloScene;
use crate::render::prepare::PreparedAffine;
use crate::render::{
    SkipEncoding, SkipScaling, VelloEntityCountData, VelloScreenScale, VelloView, VelloWorldScale,
};

#[derive(Component, Clone)]
pub struct ExtractedWorldVelloScene {
    pub scene: VelloScene,
    pub transform: GlobalTransform,
    pub render_space: VelloRenderSpace,
    pub skip_scaling: Option<SkipScaling>,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloScene {
    pub scene: VelloScene,
    pub ui_transform: UiGlobalTransform,
    pub ui_node: ComputedNode,
    pub skip_scaling: Option<SkipScaling>,
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
                &VelloScene,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&RenderLayers>,
                &VelloRenderSpace,
                Option<&SkipScaling>,
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

    for (
        scene,
        transform,
        view_visibility,
        inherited_visibility,
        render_layers,
        render_space,
        skip_scaling,
    ) in query_scenes.iter()
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
                .spawn(ExtractedWorldVelloScene {
                    transform: *transform,
                    scene: scene.clone(),
                    render_space: *render_space,
                    skip_scaling: skip_scaling.cloned(),
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
                &VelloScene,
                &ComputedNode,
                &UiGlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&RenderLayers>,
                Option<&SkipScaling>,
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

    for (
        scene,
        ui_node,
        ui_transform,
        view_visibility,
        inherited_visibility,
        render_layers,
        skip_scaling,
    ) in query_scenes.iter()
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
                .spawn(ExtractedUiVelloScene {
                    scene: scene.clone(),
                    ui_transform: *ui_transform,
                    ui_node: *ui_node,
                    skip_scaling: skip_scaling.cloned(),
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
    render_entities: Query<(Entity, &ExtractedWorldVelloScene)>,
    render_ui_entities: Query<(Entity, &ExtractedUiVelloScene)>,
    world_scale: Res<VelloWorldScale>,
    screen_scale: Res<VelloScreenScale>,
) {
    let screen_scale_matrix = Mat4::from_scale(Vec3::new(screen_scale.0, screen_scale.0, 1.0));
    let world_scale_matrix = Mat4::from_scale(Vec3::new(world_scale.0, world_scale.0, 1.0));

    for (camera, view) in views.iter() {
        let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
        let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
        let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
            [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
            [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .transpose();

        for (entity, render_entity) in render_ui_entities.iter() {
            let ui_transform = render_entity.ui_transform;
            let ui_node = render_entity.ui_node;
            let needs_scaling = render_entity.skip_scaling.is_none();

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
                let mut model_matrix = Mat4::from_cols_array_2d(&[
                    [mat2.x_axis.x, mat2.x_axis.y, 0.0, 0.0],
                    [mat2.y_axis.x, mat2.y_axis.y, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [translation.x, translation.y, 0.0, 1.0],
                ]);

                // Apply node centering transformation
                let Vec2 { x, y } = ui_node.size();
                let local_center_matrix =
                    Mat4::from_translation(Vec3::new(x / 2.0, y / 2.0, 0.0)).inverse();

                if needs_scaling {
                    model_matrix *= screen_scale_matrix;
                }

                let raw_transform = model_matrix * local_center_matrix;
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
        for (entity, render_entity) in render_entities.iter() {
            let world_transform = render_entity.transform;
            let needs_scaling = render_entity.skip_scaling.is_none();

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
            let transform: [f64; 6] = match render_entity.render_space {
                VelloRenderSpace::World => {
                    let mut model_matrix = world_transform.to_matrix();

                    if needs_scaling {
                        model_matrix *= world_scale_matrix;
                    }

                    // Flip Y-axis to match Vello's y-down coordinate space
                    model_matrix.w_axis.y *= -1.0;

                    let (projection_mat, view_mat) = {
                        let mut view_mat = view.world_from_view.to_matrix();

                        // Flip Y-axis to match Vello's y-down coordinate space
                        view_mat.w_axis.y *= -1.0;

                        (view.clip_from_view, view_mat)
                    };
                    let view_proj_matrix = projection_mat * view_mat.inverse();

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
                }
                VelloRenderSpace::Screen => {
                    let mut model_matrix = world_transform.to_matrix();

                    if needs_scaling {
                        model_matrix *= screen_scale_matrix;
                    }

                    let raw_transform = model_matrix;
                    let transform = raw_transform.to_cols_array();
                    [
                        transform[0] as f64,  // a // scale_x
                        transform[1] as f64,  // b // skew_y
                        transform[4] as f64,  // c // skew_x
                        transform[5] as f64,  // d // scale_y
                        transform[12] as f64, // e // translate_x
                        transform[13] as f64, // f // translate_y
                    ]
                }
            };

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}
