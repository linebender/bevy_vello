#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] -- This would be great! But we are far away.
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use crate::prelude::*;
use bevy::prelude::*;

mod plugin;
pub use plugin::VelloPlugin;

pub mod debug;
pub mod diagnostics;
pub mod integrations;
pub mod render;
pub mod text;

// Re-exports
pub use velato;
pub use vello;
pub use vello_svg;

pub mod prelude {
    pub use skrifa;
    pub use vello::{self, kurbo, peniko};

    pub use crate::{
        debug::DebugVisualizations,
        render::{SkipEncoding, VelloRenderSettings, VelloView},
        text::{VelloFont, VelloTextAnchor, VelloTextSection, VelloTextStyle},
        CoordinateSpace, VelloScene, VelloSceneBundle, VelloTextBundle,
    };

    #[cfg(feature = "lottie")]
    pub use crate::integrations::lottie::{
        LottieExt, LottiePlayer, PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions,
        PlaybackPlayMode, PlayerState, PlayerTransition, Playhead, Theme, VelloLottie,
        VelloLottieAnchor, VelloLottieBundle, VelloLottieHandle,
    };
    #[cfg(feature = "svg")]
    pub use crate::integrations::svg::{VelloSvg, VelloSvgAnchor, VelloSvgBundle, VelloSvgHandle};
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
pub struct VelloSceneBundle {
    /// Scene to render
    pub scene: VelloScene,
    /// The coordinate space in which this scene should be rendered.
    pub coordinate_space: CoordinateSpace,
    /// A transform to apply to this scene
    pub transform: Transform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
}

#[derive(Bundle, Default)]
pub struct VelloTextBundle {
    /// Text to render
    pub text: VelloTextSection,
    /// How the text is positioned relative to its [`Transform`].
    pub text_anchor: VelloTextAnchor,
    /// The coordinate space in which this text should be rendered.
    pub coordinate_space: CoordinateSpace,
    /// A transform to apply to this text
    pub transform: Transform,
    /// Whether to render debug visualizations
    pub debug_visualizations: DebugVisualizations,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
}

/// A simple newtype component wrapper for [`vello::Scene`] for rendering.
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(CoordinateSpace, Transform, Visibility)]
pub struct VelloScene(vello::Scene);

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
