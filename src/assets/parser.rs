use crate::{
    assets::vector::Vector,
    lyon_utils::{self, usvg_draw, Convert},
    VelloVector,
};
use bevy::prelude::*;
use lyon_tessellation::{FillTessellator, StrokeTessellator};
use std::sync::Arc;
use vello::{SceneBuilder, SceneFragment};
use vello_svg::usvg;

/// Deserialize the SVG source XML string from the file
/// contents buffer represented as raw bytes into a `VelloVector`
pub fn load_svg_from_bytes(bytes: &[u8]) -> Result<VelloVector, bevy::asset::Error> {
    let svg_str = std::str::from_utf8(bytes)?;

    let usvg = usvg::Tree::from_str(svg_str, &usvg::Options::default())?;

    // Process the loaded SVG into Vello-compatible data
    let mut scene_frag = SceneFragment::new();
    let mut builder = SceneBuilder::for_fragment(&mut scene_frag);
    vello_svg::render_tree(&mut builder, &usvg);

    // Load SVG XML String with the USVG parser
    let lyon_svg = usvg_draw::Svg::from_tree(&usvg);
    let tessellation_mesh_buffer = lyon_utils::generate_buffer(
        &lyon_svg,
        &mut FillTessellator::new(),
        &mut StrokeTessellator::new(),
    );

    let tessellation_mesh: Mesh = tessellation_mesh_buffer.convert();

    let width = usvg.size.width() as f32;
    let height = usvg.size.height() as f32;

    let vello_vector = VelloVector {
        data: Vector::Static(Arc::new(scene_frag)),
        local_transform: compute_local_transform(width, height),
        width,
        height,
        tessellation_mesh: Some(tessellation_mesh),
    };

    Ok(vello_vector)
}

/// Deserialize the Lottie source JSON string from the file
/// contents buffer represented as string into a `VelloVector`
pub fn load_svg_from_str(svg_str: &str) -> Result<VelloVector, bevy::asset::Error> {
    let bytes = svg_str.as_bytes();

    load_svg_from_bytes(bytes)
}

/// Deserialize the Lottie source JSON string from the file
/// contents buffer represented as a str into a `VelloVector`
pub fn load_lottie_from_bytes(bytes: &[u8]) -> Result<VelloVector, bevy::asset::Error> {
    // Load Lottie JSON bytes with the Velato (bodymovin) parser
    let composition = velato::Composition::from_bytes(bytes)
        .map_err(|err| bevy::asset::Error::msg(format!("Unable to parse lottie JSON: {err:?}")))?;

    let width = composition.width as f32;
    let height = composition.height as f32;

    let vello_vector = VelloVector {
        data: Vector::Animated(composition),
        local_transform: compute_local_transform(width, height),
        width,
        height,
        tessellation_mesh: None,
    };

    Ok(vello_vector)
}

/// Deserialize the Lottie source JSON string from the file
/// contents buffer represented as a str into a `VelloVector`
pub fn load_lottie_from_str(json_str: &str) -> Result<VelloVector, bevy::asset::Error> {
    let bytes = json_str.as_bytes();

    load_lottie_from_bytes(bytes)
}

fn compute_local_transform(width: f32, height: f32) -> Transform {
    let mut transform = Transform::default();
    transform.translation.x = width / 2.0;
    transform.translation.y = -height;

    transform
}
