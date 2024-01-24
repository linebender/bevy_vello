use super::extract::{ExtractedPixelScale, ExtractedRenderText, ExtractedRenderVector};
use crate::{assets::vector::VelloAsset, CoordinateSpace, Vector};
use bevy::{
    prelude::*,
    render::{camera::ExtractedCamera, render_asset::RenderAssets, view::ExtractedView},
};
use vello::kurbo::Affine;

#[derive(Component, Copy, Clone)]
pub struct PreparedAffine(pub Affine);

pub fn prepare_vector_affines(
    mut commands: Commands,
    camera: Query<(&ExtractedCamera, &ExtractedView)>,
    mut render_vectors: Query<(Entity, &ExtractedRenderVector)>,
    render_vector_assets: Res<RenderAssets<VelloAsset>>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    let Ok((camera, view)) = camera.get_single() else {
        return;
    };
    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
    let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
    for (entity, render_vector) in render_vectors.iter_mut() {
        let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
            [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
            [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .transpose();

        let world_transform = render_vector.transform;

        let local_bottom_center_matrix = render_vector
            .asset
            .local_transform_bottom_center
            .compute_matrix()
            .inverse();
        let local_center_matrix = render_vector
            .asset
            .local_transform_center
            .compute_matrix()
            .inverse();
        let vector_size = Vec2::new(render_vector.asset.width, render_vector.asset.height);

        let raw_transform = match render_vector.render_mode {
            CoordinateSpace::ScreenSpace => {
                let mut model_matrix = world_transform.compute_matrix().mul_scalar(pixel_scale.0);

                // Make the screen space vector instance sized to fill the entire UI Node box if it's bundled with a Node
                if let Some(node) = &render_vector.ui_node {
                    let fill_scale = node.size() / vector_size;
                    model_matrix.x_axis.x *= fill_scale.x;
                    model_matrix.y_axis.y *= fill_scale.y;
                }

                let mut local_center_matrix = local_center_matrix;
                local_center_matrix.w_axis.y *= -1.0;
                model_matrix * local_center_matrix
            }
            CoordinateSpace::WorldSpace => {
                let local_matrix = match render_vector.origin {
                    crate::Origin::BottomCenter => local_bottom_center_matrix,
                    crate::Origin::Center => local_center_matrix,
                };

                let mut model_matrix = world_transform.compute_matrix() * local_matrix;
                model_matrix.w_axis.y *= -1.0;

                let (projection_mat, view_mat) = {
                    let mut view_mat = view.transform.compute_matrix();
                    view_mat.w_axis.y *= -1.0;

                    (view.projection, view_mat)
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
    camera: Query<(&ExtractedCamera, &ExtractedView)>,
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
            let mut view_mat = view.transform.compute_matrix();
            view_mat.w_axis.y *= -1.0;

            (view.projection, view_mat)
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

pub fn prepare_recoloring(mut render_vectors: Query<&mut ExtractedRenderVector>) {
    for mut render_vector in render_vectors.iter_mut() {
        // Get vector and color swap or there's no use continuing...
        let ExtractedRenderVector {
            asset, color_swaps, ..
        } = render_vector.as_mut();

        // Perform recolors!
        // TODO: Recoloring SVGs
        let Vector::Lottie {
            ref original,
            ref mut colored,
            ..
        } = asset.data
        else {
            continue;
        };

        // Continue if there are no colors
        let Some(color_swaps) = color_swaps else {
            colored.take();
            continue;
        };

        let colored_composition = color_swaps.create(original);
        colored.replace(colored_composition);
    }
}
