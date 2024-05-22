use crate::{integrations::VectorLoaderError, VectorFile, VelloAsset};
use bevy::transform::components::Transform;
use once_cell::sync::Lazy;
use std::sync::Arc;
use vello_svg::usvg::{self, fontdb::Database};

pub static FONT_DB: Lazy<Database> = Lazy::new(usvg::fontdb::Database::default);

/// Deserialize an SVG file from bytes.
pub fn load_svg_from_bytes(bytes: &[u8]) -> Result<VelloAsset, VectorLoaderError> {
    let svg_str = std::str::from_utf8(bytes)?;

    let usvg = usvg::Tree::from_str(svg_str, &usvg::Options::default(), &FONT_DB)?;

    // Process the loaded SVG into Vello-compatible data
    let mut scene = vello::Scene::new();
    vello_svg::render_tree(&mut scene, &usvg);

    let width = usvg.size().width();
    let height = usvg.size().height();

    let vello_vector = VelloAsset {
        file: VectorFile::Svg(Arc::new(scene)),
        local_transform_center: {
            let mut transform = Transform::default();
            transform.translation.x = width / 2.0;
            transform.translation.y = -height / 2.0;
            transform
        },
        width,
        height,
        alpha: 1.0,
    };

    Ok(vello_vector)
}

/// Deserialize an SVG file from a string slice.
pub fn load_svg_from_str(svg_str: &str) -> Result<VelloAsset, VectorLoaderError> {
    let bytes = svg_str.as_bytes();

    load_svg_from_bytes(bytes)
}
