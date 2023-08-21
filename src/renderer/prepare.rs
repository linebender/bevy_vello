use bevy::{
    prelude::*,
    render::{camera::ExtractedCamera, render_asset::RenderAssets, view::ExtractedView},
};
use vello::kurbo::Affine;

use crate::{
    assets::vector::{Vector, VelloVector},
    ColorPaletteSwap, Layer,
};

use super::extract::{ExtractedPixelScale, ExtractedRenderText, ExtractedRenderVector};

pub fn prepare_vector_composition_edits(mut render_vectors: Query<&mut ExtractedRenderVector>) {
    // Big-O: O(n), where n = shapes;
    // Nesting: "vectors * layers * shape groups * shapes"
    'vectors: for mut render_vector in render_vectors.iter_mut() {
        // Get vector and color swap or there's no use continuing...

        let Some(ColorPaletteSwap { colors }) = render_vector.color_pallette_swap.clone() else {
            continue 'vectors;
        };

        // Perform recolors!
        let Vector::Animated(ref mut composition) = render_vector.render_data.data else {
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

                    for ((layer_name, shape_indices), color) in colors.iter() {
                        if layer.name.contains(layer_name) && shape_indices.contains(&shape_index) {
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

#[derive(Component, Copy, Clone)]
pub struct PreparedAffine(pub Affine);

pub fn prepare_vector_affines(
    mut commands: Commands,
    camera: Query<(&ExtractedCamera, &ExtractedView)>,
    mut render_vectors: Query<(Entity, &ExtractedRenderVector)>,
    render_vector_assets: Res<RenderAssets<VelloVector>>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    let Ok((camera, view)) = camera.get_single() else { return };
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
        let (local_bottom_center_matrix, local_center_matrix, vector_size) =
            match render_vector_assets.get(&render_vector.vector_handle) {
                Some(render_instance_data) => (
                    render_instance_data.local_bottom_center_matrix,
                    render_instance_data.local_center_matrix,
                    render_instance_data.size,
                ),
                None => continue,
            };

        // The vello scene transform is world-space for all normal vectors and screen-space for UI vectors
        let raw_transform = match render_vector.layer {
            Layer::UI => {
                let mut model_matrix = world_transform.compute_matrix().mul_scalar(pixel_scale.0);

                // Make the UI vector instance sized to fill the entire UI Node box if it's bundled with a Node
                if let Some(node) = &render_vector.ui_node {
                    let fill_scale = node.size() / vector_size;
                    model_matrix.x_axis.x *= fill_scale.x;
                    model_matrix.y_axis.y *= fill_scale.y;
                }

                model_matrix * local_center_matrix
            }
            _ => {
                let mut model_matrix =
                    world_transform.compute_matrix() * local_bottom_center_matrix;
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
    let Ok((camera, view)) = camera.get_single() else { return };
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

        let raw_transform = match render_text.layer {
            Layer::UI => world_transform.compute_matrix().mul_scalar(pixel_scale.0),
            _ => vello_matrix * model_matrix,
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
