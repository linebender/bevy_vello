mod asset_loader;
mod systems;

pub(crate) mod render;

mod asset;
pub use asset::VelloSvg;

mod parse;
pub use parse::{load_svg_from_bytes, load_svg_from_str};

mod plugin;
pub(crate) use plugin::SvgIntegrationPlugin;

use bevy::{
    camera::{primitives::Aabb, visibility::VisibilityClass},
    prelude::*,
};

/// A renderable SVG in the world.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(Aabb, VelloSvgAnchor, Transform, Visibility, VisibilityClass)]
#[cfg_attr(feature = "picking", require(Pickable))]
#[reflect(Component)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<VelloSvg2d>)]
pub struct VelloSvg2d(pub Handle<VelloSvg>);

/// A renderable SVG that may be used in Bevy UI.
///
/// ### Object fit
/// The image will preserve the aspect ratio, and fits the image inside the container, without cutting - will leave empty space if needed.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(Node, VelloSvgAnchor, Visibility, VisibilityClass)]
#[reflect(Component)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<UiVelloSvg>)]
pub struct UiVelloSvg(pub Handle<VelloSvg>);

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
