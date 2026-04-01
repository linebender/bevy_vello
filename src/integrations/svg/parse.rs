use std::sync::Arc;

use super::asset::VelloSvg;
use crate::integrations::VectorLoaderError;

/// Deserialize an SVG file from bytes.
pub fn load_svg_from_bytes(bytes: &[u8]) -> Result<VelloSvg, VectorLoaderError> {
    let mut options = svg_imaging::ParseOptions::default();
    options.fontdb_mut().load_system_fonts();
    let document = svg_imaging::SvgDocument::from_data(bytes, &options)?;
    let mut scene = imaging::record::Scene::new();
    {
        let mut painter = imaging::Painter::new(&mut scene);
        document.render(&mut painter, &svg_imaging::RenderOptions::default())?;
    }

    let size = document.size();
    let width = size.width as f32;
    let height = size.height as f32;

    let asset = VelloSvg {
        scene: Arc::new(scene),
        width,
        height,
        alpha: 1.0,
    };

    Ok(asset)
}

/// Deserialize an SVG file from a string slice.
pub fn load_svg_from_str(svg_str: &str) -> Result<VelloSvg, VectorLoaderError> {
    let bytes = svg_str.as_bytes();

    load_svg_from_bytes(bytes)
}
