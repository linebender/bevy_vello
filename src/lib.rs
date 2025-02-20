#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] -- This would be great! But we are far away.
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use crate::prelude::*;
use bevy::prelude::*;

mod plugin;
pub use plugin::VelloPlugin;

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
        render::{SkipEncoding, VelloRenderSettings, VelloView},
        text::{VelloFont, VelloTextAnchor, VelloTextSection, VelloTextStyle},
        VelloScene, VelloSceneBundle, VelloTextBundle,
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

#[derive(Bundle, Default)]
pub struct VelloSceneBundle {
    /// Scene to render
    pub scene: VelloScene,
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
    /// A transform to apply to this text
    pub transform: Transform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
}

/// A simple newtype component wrapper for [`vello::Scene`] for rendering.
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(Transform, Visibility)]
pub struct VelloScene(Box<vello::Scene>);

impl VelloScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<vello::Scene> for VelloScene {
    fn from(scene: vello::Scene) -> Self {
        Self(Box::new(scene))
    }
}
