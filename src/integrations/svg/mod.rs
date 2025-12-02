mod asset_loader;

pub(crate) mod render;

mod asset;
pub use asset::{VelloSvg, VelloSvgHandle};

mod parse;
pub use parse::{load_svg_from_bytes, load_svg_from_str};

mod plugin;
use bevy::{camera::visibility::VisibilityClass, prelude::*};
pub(crate) use plugin::SvgIntegrationPlugin;

#[derive(Bundle, Default)]
pub struct VelloSvgBundle {
    /// Asset data to render
    pub asset: VelloSvgHandle,
    /// How the asset is positioned relative to its [`Transform`].
    pub asset_anchor: VelloSvgAnchor,
    /// A transform to apply to this vector
    pub transform: Transform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub view_visibility: Visibility,
    /// A bucket into which we group entities for the purposes of visibility.
    pub visibility_class: VisibilityClass,
}

/// Describes how the asset is positioned relative to its [`Transform`]. It defaults to
/// [`VelloSvgAnchor::Center`].
///
/// Has no effect in UI nodes.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum VelloSvgAnchor {
    /// Bounds start from the render position and advance up and to the right.
    BottomLeft,
    /// Bounds start from the render position and advance up.
    Bottom,
    /// Bounds start from the render position and advance up and to the left.
    BottomRight,

    /// Bounds start from the render position and advance right.
    Left,
    /// Bounds start from the render position and advance equally on both axes.
    #[default]
    Center,
    /// Bounds start from the render position and advance left.
    Right,

    /// Bounds start from the render position and advance down and to the right.
    TopLeft,
    /// Bounds start from the render position and advance down.
    Top,
    /// Bounds start from the render position and advance down and to the left.
    TopRight,
}
