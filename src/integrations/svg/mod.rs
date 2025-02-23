mod asset_loader;

pub(crate) mod render;

mod asset;
pub use asset::{VelloSvg, VelloSvgHandle};

mod parse;
pub use parse::{load_svg_from_bytes, load_svg_from_str};

mod plugin;
pub(crate) use plugin::SvgIntegrationPlugin;

use crate::debug::DebugVisualizations;
use bevy::prelude::*;
#[derive(Bundle, Default)]
pub struct VelloSvgBundle {
    /// Asset data to render
    pub asset: VelloSvgHandle,
    /// How the asset is positioned relative to its [`Transform`].
    pub asset_anchor: VelloSvgAnchor,
    /// A transform to apply to this vector
    pub transform: Transform,
    /// Whether to render debug visualizations
    pub debug_visualizations: DebugVisualizations,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
}

/// Describes how the asset is positioned relative to its [`Transform`]. It defaults to [`VelloAssetAnchor::Center`].
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
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

impl VelloSvgAnchor {
    pub(crate) fn compute(
        &self,
        width: f32,
        height: f32,
        transform: &GlobalTransform,
    ) -> GlobalTransform {
        // Apply positioning
        let adjustment = match self {
            Self::TopLeft => Vec3::new(width / 2.0, -height / 2.0, 0.0),
            Self::Left => Vec3::new(width / 2.0, 0.0, 0.0),
            Self::BottomLeft => Vec3::new(width / 2.0, height / 2.0, 0.0),
            Self::Top => Vec3::new(0.0, -height / 2.0, 0.0),
            Self::Center => Vec3::new(0.0, 0.0, 0.0),
            Self::Bottom => Vec3::new(0.0, height / 2.0, 0.0),
            Self::TopRight => Vec3::new(-width / 2.0, -height / 2.0, 0.0),
            Self::Right => Vec3::new(-width / 2.0, 0.0, 0.0),
            Self::BottomRight => Vec3::new(-width / 2.0, height / 2.0, 0.0),
        };
        let new_translation: Vec3 = (transform.compute_matrix() * adjustment.extend(1.0)).xyz();
        GlobalTransform::from(
            transform
                .compute_transform()
                .with_translation(new_translation),
        )
    }
}
