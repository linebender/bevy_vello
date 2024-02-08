use super::asset_loader::VectorLoaderError;
use crate::{assets::vector::VelloAssetData, VelloAsset};
use bevy::prelude::*;
use std::sync::Arc;
use vello::{SceneBuilder, SceneFragment};
use vello_svg::usvg::{self, TreeParsing};

/// Deserialize an SVG file from bytes.
pub fn load_svg_from_bytes(bytes: &[u8]) -> Result<VelloAsset, VectorLoaderError> {
    let svg_str = std::str::from_utf8(bytes)?;

    let usvg = usvg::Tree::from_str(svg_str, &usvg::Options::default())?;

    // Process the loaded SVG into Vello-compatible data
    let mut scene_frag = SceneFragment::new();
    let mut builder = SceneBuilder::for_fragment(&mut scene_frag);
    vello_svg::render_tree(&mut builder, &usvg, None);

    let width = usvg.size.width();
    let height = usvg.size.height();

    let vello_vector = VelloAsset {
        data: VelloAssetData::Svg {
            original: Arc::new(scene_frag),
        },
        local_transform_center: compute_local_transform_center(width, height),
        width,
        height,
    };

    Ok(vello_vector)
}

/// Deserialize an SVG file from a string slice.
pub fn load_svg_from_str(svg_str: &str) -> Result<VelloAsset, VectorLoaderError> {
    let bytes = svg_str.as_bytes();

    load_svg_from_bytes(bytes)
}

/// Deserialize a Lottie file from bytes.
pub fn load_lottie_from_bytes(bytes: &[u8]) -> Result<VelloAsset, VectorLoaderError> {
    // Load Lottie JSON bytes with the Velato (bodymovin) parser
    let composition = vellottie::Composition::from_bytes(bytes)
        .map_err(|err| VectorLoaderError::Parse(format!("Unable to parse lottie JSON: {err:?}")))?;

    let width = composition.width as f32;
    let height = composition.height as f32;

    let vello_vector = VelloAsset {
        data: VelloAssetData::Lottie {
            composition: Arc::new(composition),
        },
        local_transform_center: compute_local_transform_center(width, height),
        width,
        height,
    };

    Ok(vello_vector)
}

/// Deserialize a Lottie file from a string slice.
pub fn load_lottie_from_str(json_str: &str) -> Result<VelloAsset, VectorLoaderError> {
    let bytes = json_str.as_bytes();

    load_lottie_from_bytes(bytes)
}

fn compute_local_transform_center(width: f32, height: f32) -> Transform {
    let mut transform = Transform::default();
    transform.translation.x = width / 2.0;
    transform.translation.y = -height / 2.0;

    transform
}
