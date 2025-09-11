use bevy::{
    prelude::*,
    render::{camera::ExtractedCamera, view::ExtractedView},
};
use vello::kurbo::Affine;

use super::{VelloScreenScale, VelloView, VelloWorldScale, extract::ExtractedVelloScene};

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedAffine(pub Affine);

#[allow(dead_code)]
#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedTransform(pub GlobalTransform);

// All extracted bevy_vello render instance types should implement this (RenderAsset, RenderScene,
// RenderText, etc...)
#[cfg(any(feature = "svg", feature = "lottie"))]
pub trait PrepareRenderInstance {
    fn final_transform(&self) -> PreparedTransform;
    fn scene_affine(
        &self,
        view: &ExtractedView,
        world_transform: GlobalTransform,
        viewport_size: UVec2,
        world_scale: f32,
        screen_scale: f32,
    ) -> PreparedAffine;
}

pub fn prepare_scene_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView), (With<Camera2d>, With<VelloView>)>,
    render_entities: Query<(Entity, &ExtractedVelloScene)>,
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

        for (entity, render_entity) in render_entities.iter() {
            let world_transform = render_entity.transform;
            let is_scaled = render_entity.skip_scaling.is_none();

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
            let transform: [f64; 6] = if let Some(node) = render_entity.ui_node {
                let mut model_matrix = world_transform.compute_matrix();
                let Vec2 { x, y } = node.size();
                let local_center_matrix =
                    Mat4::from_translation(Vec3::new(x / 2.0, y / 2.0, 0.0)).inverse();

                if is_scaled {
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
            } else if render_entity.screen_space.is_some() {
                let mut model_matrix = world_transform.compute_matrix();

                if is_scaled {
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
            } else {
                let mut model_matrix = world_transform.compute_matrix();

                if is_scaled {
                    model_matrix *= world_scale_matrix;
                }

                // Flip Y-axis to match Vello's y-down coordinate space
                model_matrix.w_axis.y *= -1.0;

                let (projection_mat, view_mat) = {
                    let mut view_mat = view.world_from_view.compute_matrix();

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
            };

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}
