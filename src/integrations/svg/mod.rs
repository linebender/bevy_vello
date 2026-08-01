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

use super::VelloAnchor;

/// A renderable SVG in the world.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(Aabb, Transform, VelloAnchor, Visibility, VisibilityClass)]
#[cfg_attr(feature = "picking", require(Pickable))]
#[reflect(Component)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<VelloSvg2d>)]
pub struct VelloSvg2d(pub Handle<VelloSvg>);

/// A renderable SVG that may be used in Bevy UI.
///
/// ### Object fit
/// The image will preserve the aspect ratio, and fits the image inside the container, without cutting - will leave empty space if needed.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(Node, VelloAnchor, Visibility, VisibilityClass)]
#[reflect(Component)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<UiVelloSvg>)]
pub struct UiVelloSvg(pub Handle<VelloSvg>);
