mod asset_loader;

pub(crate) mod render;

mod asset;
pub use asset::{VelloSvg, VelloSvgHandle};

mod parse;
pub use parse::{load_svg_from_bytes, load_svg_from_str};

mod plugin;
pub(crate) use plugin::SvgIntegrationPlugin;

use super::VelloAssetAnchor;
use crate::{debug::DebugVisualizations, CoordinateSpace};
use bevy::prelude::*;
#[derive(Bundle, Default)]
pub struct VelloSvgBundle {
    /// Asset data to render
    pub asset: VelloSvgHandle,
    /// How the asset is positioned relative to its [`Transform`].
    pub asset_anchor: VelloAssetAnchor,
    /// The coordinate space in which this vector should be rendered.
    pub coordinate_space: CoordinateSpace,
    /// A transform to apply to this vector
    pub transform: Transform,
    /// Whether to render debug visualizations
    pub debug_visualizations: DebugVisualizations,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
}
