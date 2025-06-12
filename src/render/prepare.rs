use bevy::{
    prelude::*,
    render::{camera::ExtractedCamera, view::ExtractedView},
};
use vello::kurbo::Affine;

use super::{VelloScreenScale, VelloView, VelloWorldScale, extract::ExtractedVelloScene};

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedAffine(pub Affine);

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
    for (camera, view) in views.iter() {
        let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
        let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);

        // Render scenes
        for (entity, render_entity) in render_entities.iter() {
            let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
                [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .transpose();

            let world_transform = render_entity.transform;

            let raw_transform = if let Some(node) = &render_entity.ui_node {
                let mut model_matrix = world_transform.compute_matrix();

                // The Bevy Transform for a UI node seems to always have the origin
                // of the translation at the center of its bounding box. Here we
                // move the origin back to the top left, so that, e.g., drawing a
                // shape with center=(20,20) inside of a 40x40 UI node results in
                // the shape being centered within the node.
                let Vec2 { x, y } = node.size();
                model_matrix.w_axis.x -= x * 0.5;
                model_matrix.w_axis.y -= y * 0.5;

                // Note that there's no need to flip the Y axis in this case, as
                // Bevy handles it for us.
                model_matrix
            } else if render_entity.screen_space.is_some() {
                let mut model_matrix = world_transform.compute_matrix();
                if render_entity.no_scaling.is_none() {
                    model_matrix.x_axis.x *= screen_scale.0;
                    model_matrix.y_axis.y *= screen_scale.0;
                }
                model_matrix
            } else {
                let mut model_matrix = world_transform.compute_matrix();
                model_matrix.w_axis.y *= -1.0;

                if render_entity.no_scaling.is_none() {
                    model_matrix.x_axis.x *= world_scale.0;
                    model_matrix.y_axis.y *= world_scale.0;
                }

                let (projection_mat, view_mat) = {
                    let mut view_mat = view.world_from_view.compute_matrix();
                    view_mat.w_axis.y *= -1.0;

                    (view.clip_from_view, view_mat)
                };

                let view_proj_matrix = projection_mat * view_mat.inverse();

                ndc_to_pixels_matrix * view_proj_matrix * model_matrix
            };

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

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}
