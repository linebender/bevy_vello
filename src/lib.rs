#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] -- This would be great! But we are far away.
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use crate::prelude::*;
use bevy::prelude::*;

mod plugin;
pub use plugin::VelloPlugin;

pub mod debug;
pub mod integrations;
pub mod render;
pub mod text;

// Re-exports
pub use velato;
pub use vello;
pub use vello_svg;

pub mod prelude {
    pub use vello::{self, kurbo, peniko, skrifa};

    pub use crate::{
        debug::DebugVisualizations,
        integrations::{VectorFile, VelloAsset, VelloAssetAnchor},
        render::VelloCanvasMaterial,
        text::{VelloFont, VelloText, VelloTextAnchor},
        CoordinateSpace, VelloAssetBundle, VelloScene, VelloSceneBundle, VelloTextBundle,
    };

    #[cfg(feature = "experimental-dotLottie")]
    pub use crate::integrations::dot_lottie::{DotLottiePlayer, PlayerState, PlayerTransition};
    #[cfg(feature = "lottie")]
    pub use crate::integrations::lottie::{
        LottieExt, PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions, PlaybackPlayMode,
        Playhead, Theme,
    };
}

/// Which coordinate space the transform is relative to.
#[derive(PartialEq, Eq, PartialOrd, Ord, Component, Default, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum CoordinateSpace {
    #[default]
    WorldSpace,
    ScreenSpace,
}

#[derive(Bundle, Default)]
pub struct VelloAssetBundle {
    /// Asset data to render
    pub asset: Handle<VelloAsset>,
    /// How the asset is positioned relative to its [`Transform`].
    pub asset_anchor: VelloAssetAnchor,
    /// The coordinate space in which this vector should be rendered.
    pub coordinate_space: CoordinateSpace,
    /// A transform to apply to this vector
    pub transform: Transform,
    /// The global transform managed by Bevy
    pub global_transform: GlobalTransform,
    /// Whether to render debug visualizations
    pub debug_visualizations: DebugVisualizations,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
    /// Whether or not an entity is visible in the hierarchy.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible. Should be extracted
    /// for rendering.
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
pub struct VelloSceneBundle {
    /// Scene to render
    pub scene: VelloScene,
    /// The coordinate space in which this scene should be rendered.
    pub coordinate_space: CoordinateSpace,
    /// A transform to apply to this scene
    pub transform: Transform,
    /// The global transform managed by Bevy
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
    /// Whether or not an entity is visible in the hierarchy.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible. Should be extracted
    /// for rendering.
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
pub struct VelloTextBundle {
    /// Font to render
    pub font: Handle<VelloFont>,
    /// Text to render
    pub text: VelloText,
    /// How the text is positioned relative to its [`Transform`].
    pub text_anchor: VelloTextAnchor,
    /// The coordinate space in which this text should be rendered.
    pub coordinate_space: CoordinateSpace,
    /// A transform to apply to this text
    pub transform: Transform,
    /// The global transform managed by Bevy
    pub global_transform: GlobalTransform,
    /// Whether to render debug visualizations
    pub debug_visualizations: DebugVisualizations,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
    /// Whether or not an entity is visible in the hierarchy.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible. Should be extracted
    /// for rendering.
    pub view_visibility: ViewVisibility,
}

/// A simple newtype component wrapper for [`vello::Scene`] for rendering.
#[derive(Component, Default, Clone)]
pub struct VelloScene(vello::Scene);

impl std::ops::Deref for VelloScene {
    type Target = vello::Scene;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for VelloScene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl VelloScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<vello::Scene> for VelloScene {
    fn from(scene: vello::Scene) -> Self {
        Self(scene)
    }
}
