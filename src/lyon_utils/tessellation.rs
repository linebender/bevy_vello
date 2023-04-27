use crate::lyon_utils::{
    usvg_draw::{DrawType, Svg},
    vertex_buffer::{BufferExt, VertexBuffers, VertexConstructor},
};
use bevy::prelude::*;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeTessellator,
};

pub fn generate_buffer(
    svg: &Svg,
    fill_tess: &mut FillTessellator,
    stroke_tess: &mut StrokeTessellator,
) -> VertexBuffers {
    debug!("Tessellating SVG: {}", svg.name);

    let flip_y = Transform::from_scale(Vec3::new(1.0, -1.0, 1.0));
    let mut buffers = VertexBuffers::new();

    let mut color = None;
    for path in &svg.paths {
        let mut buffer = VertexBuffers::new();

        if color.is_none() {
            color = Some(path.color);
        }

        // Bevy has a different y-axis origin, so we need to flip that axis
        let transform = flip_y * path.abs_transform;
        match path.draw_type {
            DrawType::Fill => {
                if let Err(e) = fill_tess.tessellate(
                    path.segments.clone(),
                    &FillOptions::tolerance(0.1),
                    &mut BuffersBuilder::new(
                        &mut buffer,
                        VertexConstructor {
                            color: path.color,
                            transform,
                        },
                    ),
                ) {
                    error!("FillTessellator error: {:?}", e)
                }
                buffers.extend_one(buffer);
            }
            DrawType::Stroke(opts) => {
                if let Err(e) = stroke_tess.tessellate(
                    path.segments.clone(),
                    &opts,
                    &mut BuffersBuilder::new(
                        &mut buffer,
                        VertexConstructor {
                            color: path.color,
                            transform,
                        },
                    ),
                ) {
                    error!("StrokeTessellator error: {:?}", e)
                }
                buffers.extend_one(buffer);
            }
        }
    }
    debug!("Tessellating SVG: {} ... Done", svg.name);

    buffers
}
