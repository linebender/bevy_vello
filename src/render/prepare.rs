use super::extract::{
    ExtractedPixelScale, ExtractedRenderAsset, ExtractedRenderScene, ExtractedRenderText,
};
use crate::CoordinateSpace;
use bevy::{
    prelude::*,
    render::{camera::ExtractedCamera, view::ExtractedView},
};
use vello::kurbo::Affine;

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedAffine(Affine);

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedTransform(GlobalTransform);

// All extracted bevy_vello render instance types should implement this (RenderAsset, RenderScene,
// RenderText, etc...)
pub trait PrepareRenderInstance {
    fn final_transform(&self) -> PreparedTransform;
    fn scene_affine(
        &self,
        view: &ExtractedView,
        world_transform: GlobalTransform,
        pixel_scale: f32,
        viewport_size: UVec2,
    ) -> PreparedAffine;
}

impl PrepareRenderInstance for ExtractedRenderAsset {
    fn final_transform(&self) -> PreparedTransform {
        PreparedTransform(self.asset_anchor.compute(&self.asset, &self.transform))
    }

    fn scene_affine(
        &self,
        view: &ExtractedView,
        world_transform: GlobalTransform,
        pixel_scale: f32,
        viewport_size: UVec2,
    ) -> PreparedAffine {
        let local_center_matrix = self.asset.local_transform_center.compute_matrix().inverse();

        let raw_transform = match self.render_mode {
            CoordinateSpace::ScreenSpace => {
                let mut model_matrix = world_transform.compute_matrix().mul_scalar(pixel_scale);

                let asset_size = Vec2::new(self.asset.width, self.asset.height);

                // Make the screen space vector instance sized to fill the
                // entire UI Node box if it's bundled with a Node
                if let Some(node) = &self.ui_node {
                    let fill_scale = node.size() / asset_size;
                    model_matrix.x_axis.x *= fill_scale.x;
                    model_matrix.y_axis.y *= fill_scale.y;
                }

                let mut local_center_matrix = local_center_matrix;
                local_center_matrix.w_axis.y *= -1.0;
                model_matrix * local_center_matrix
            }
            CoordinateSpace::WorldSpace => {
                let local_matrix = local_center_matrix;

                let (pixels_x, pixels_y) = (viewport_size.x as f32, viewport_size.y as f32);
                let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
                    [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                    [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ])
                .transpose();

                let mut model_matrix = world_transform.compute_matrix() * local_matrix;
                model_matrix.w_axis.y *= -1.0;

                let (projection_mat, view_mat) = {
                    let mut view_mat = view.world_from_view.compute_matrix();
                    view_mat.w_axis.y *= -1.0;

                    (view.clip_from_view, view_mat)
                };

                let view_proj_matrix = projection_mat * view_mat.inverse();

                ndc_to_pixels_matrix * view_proj_matrix * model_matrix
            }
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

        PreparedAffine(Affine::new(transform))
    }
}

pub fn prepare_vector_affines(
    mut commands: Commands,
    camera: Query<(&ExtractedCamera, &ExtractedView), With<Camera2d>>,
    mut render_assets: Query<(Entity, &ExtractedRenderAsset)>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    let Ok((camera, view)) = camera.get_single() else {
        return;
    };
    let viewport_size: UVec2 = camera.physical_viewport_size.unwrap();
    for (entity, render_asset) in render_assets.iter_mut() {
        // Prepare render data needed for the subsequent render system
        let final_transform = render_asset.final_transform();
        let affine =
            render_asset.scene_affine(view, *final_transform, pixel_scale.0, viewport_size);

        commands.entity(entity).insert((affine, final_transform));
    }
}

pub fn prepare_scene_affines(
    mut commands: Commands,
    camera: Query<(&ExtractedCamera, &ExtractedView), With<Camera2d>>,
    mut render_scenes: Query<(Entity, &ExtractedRenderScene)>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    let Ok((camera, view)) = camera.get_single() else {
        return;
    };
    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
    let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
    for (entity, render_scene) in render_scenes.iter_mut() {
        let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
            [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
            [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .transpose();

        let world_transform = render_scene.transform;

        let raw_transform = match render_scene.render_mode {
            CoordinateSpace::ScreenSpace => {
                let mut model_matrix = world_transform.compute_matrix().mul_scalar(pixel_scale.0);

                if let Some(node) = &render_scene.ui_node {
                    // The Bevy Transform for a UI node seems to always have the origin
                    // of the translation at the center of its bounding box. Here we
                    // move the origin back to the top left, so that, e.g., drawing a
                    // shape with center=(20,20) inside of a 40x40 UI node results in
                    // the shape being centered within the node.
                    let Vec2 { x, y } = node.size() * pixel_scale.0;
                    model_matrix.w_axis.x -= x / 2.0;
                    model_matrix.w_axis.y -= y / 2.0;

                    // Note that there's no need to flip the Y axis in this case, as
                    // Bevy handles it for us.
                } else {
                    model_matrix.w_axis.y *= -1.0;
                }

                model_matrix
            }
            CoordinateSpace::WorldSpace => {
                let mut model_matrix = world_transform.compute_matrix();
                model_matrix.w_axis.y *= -1.0;

                let (projection_mat, view_mat) = {
                    let mut view_mat = view.world_from_view.compute_matrix();
                    view_mat.w_axis.y *= -1.0;

                    (view.clip_from_view, view_mat)
                };

                let view_proj_matrix = projection_mat * view_mat.inverse();

                ndc_to_pixels_matrix * view_proj_matrix * model_matrix
            }
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

pub fn prepare_text_affines(
    mut commands: Commands,
    camera: Query<(&ExtractedCamera, &ExtractedView), With<Camera2d>>,
    render_texts: Query<(Entity, &ExtractedRenderText)>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    let Ok((camera, view)) = camera.get_single() else {
        return;
    };
    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
    let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
    for (entity, render_text) in render_texts.iter() {
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
            let mut view_mat = view.world_from_view.compute_matrix();
            view_mat.w_axis.y *= -1.0;

            (view.clip_from_view, view_mat)
        };

        let view_proj_matrix = projection_mat * view_mat.inverse();
        let vello_matrix = ndc_to_pixels_matrix * view_proj_matrix;

        let raw_transform = match render_text.render_mode {
            CoordinateSpace::ScreenSpace => {
                world_transform.compute_matrix().mul_scalar(pixel_scale.0)
            }
            CoordinateSpace::WorldSpace => vello_matrix * model_matrix,
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
