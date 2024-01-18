use super::extract::{ExtractedPixelScale, ExtractedRenderText, ExtractedRenderVector};
use crate::{assets::vector::VelloVector, color_swapping::ColorPaletteSwap, RenderMode, Vector};
use bevy::{
    prelude::*,
    render::{camera::ExtractedCamera, render_asset::RenderAssets, view::ExtractedView},
};
use vello::kurbo::Affine;
use vellottie::runtime::model::Brush;

#[derive(Component, Copy, Clone)]
pub struct PreparedAffine(pub Affine);

pub fn prepare_vector_affines(
    mut commands: Commands,
    camera: Query<(&ExtractedCamera, &ExtractedView)>,
    mut render_vectors: Query<(Entity, &ExtractedRenderVector)>,
    render_vector_assets: Res<RenderAssets<VelloVector>>,
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
        let (local_bottom_center_matrix, local_center_matrix, vector_size) =
            match render_vector_assets.get(&render_vector.vector_handle) {
                Some(render_instance_data) => (
                    render_instance_data.local_bottom_center_matrix,
                    render_instance_data.local_center_matrix,
                    render_instance_data.size,
                ),
                None => continue,
            };

        let raw_transform = match render_vector.render_mode {
            RenderMode::ScreenSpace => {
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
            RenderMode::WorldSpace => {
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
            RenderMode::ScreenSpace => world_transform.compute_matrix().mul_scalar(pixel_scale.0),
            RenderMode::WorldSpace => vello_matrix * model_matrix,
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

pub fn prepare_vector_composition_edits(mut render_vectors: Query<&mut ExtractedRenderVector>) {
    // A depth-first traversal and remap of colors, O(n)
    'vectors: for mut render_vector in render_vectors.iter_mut() {
        // Get vector and color swap or there's no use continuing...
        let ExtractedRenderVector {
            render_data,
            color_swaps,
            ..
        } = render_vector.as_mut();

        // Continue if there are no colors
        let Some(ColorPaletteSwap { ref colors }) = color_swaps else {
            continue 'vectors;
        };

        // Perform recolors!
        // TODO: Recoloring SVGs
        let Vector::Animated(ref mut composition) = render_data.data else {
            continue 'vectors;
        };
        'layers: for layer in composition.layers.iter_mut() {
            // Continue if this layer doesn't have a color swap
            let Some(target_color) = colors.get(&layer.name) else {
                continue 'layers;
            };
            let shapes = match &mut layer.content {
                vellottie::runtime::model::Content::Shape(shapes) => shapes,
                vellottie::runtime::model::Content::None
                | vellottie::runtime::model::Content::Instance { .. } => {
                    continue 'layers;
                }
            };
            let target_color = vello::peniko::Color::rgba(
                target_color.r().into(),
                target_color.g().into(),
                target_color.b().into(),
                target_color.a().into(),
            );
            for shape in shapes.iter_mut() {
                match shape {
                    vellottie::runtime::model::Shape::Group(shapes, _) => {
                        for shape in shapes.iter_mut() {
                            let vellottie::runtime::model::Shape::Draw(ref mut draw) = shape else {
                                continue;
                            };
                            recolor_brush(&mut draw.brush, target_color);
                        }
                    }
                    vellottie::runtime::model::Shape::Draw(draw) => {
                        recolor_brush(&mut draw.brush, target_color);
                    }
                    vellottie::runtime::model::Shape::Repeater(_)
                    | vellottie::runtime::model::Shape::Geometry(_) => {
                        continue;
                    }
                }
            }
        }
    }
}

/// A helper method  to recolor a brush with a target color.
fn recolor_brush(brush: &mut Brush, target_color: vello::peniko::Color) {
    match brush {
        vellottie::runtime::model::Brush::Fixed(brush) => match brush {
            vello::peniko::Brush::Solid(solid) => {
                *solid = target_color;
            }
            vello::peniko::Brush::Gradient(gradient) => {
                for stop in gradient.stops.iter_mut() {
                    stop.color = target_color;
                }
            }
            vello::peniko::Brush::Image(_) => {}
        },
        vellottie::runtime::model::Brush::Animated(brush) => match brush {
            vellottie::runtime::model::animated::Brush::Solid(brush) => match brush {
                vellottie::runtime::model::Value::Fixed(solid) => {
                    *solid = target_color;
                }
                vellottie::runtime::model::Value::Animated(keyframes) => {
                    for solid in keyframes.values.iter_mut() {
                        *solid = target_color;
                    }
                }
            },
            vellottie::runtime::model::animated::Brush::Gradient(gr) => match &mut gr.stops {
                vellottie::runtime::model::ColorStops::Fixed(stops) => {
                    for stop in stops.iter_mut() {
                        stop.color = target_color;
                    }
                }
                vellottie::runtime::model::ColorStops::Animated(_stops) => {
                    // FIXME: Why does stops use f32 instead of color?
                }
            },
        },
    }
}
