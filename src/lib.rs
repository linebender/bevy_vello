#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] -- This would be great! But we are far away.
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use bevy::{
    prelude::*,
    render::view::{self, VisibilityClass},
};

use crate::prelude::*;

mod plugin;
pub use plugin::VelloPlugin;

pub mod diagnostics;
pub mod integrations;
pub mod render;

// Re-exports
#[cfg(feature = "text")]
pub use parley;
#[cfg(feature = "lottie")]
pub use velato;
pub use vello;
#[cfg(feature = "svg")]
pub use vello_svg;

pub mod prelude {
    pub use vello::{self, kurbo, peniko};

    #[cfg(feature = "lottie")]
    pub use crate::integrations::lottie::{
        LottieExt, LottiePlayer, PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions,
        PlaybackPlayMode, PlayerState, PlayerTransition, Playhead, Theme, VelloLottie,
        VelloLottieAnchor, VelloLottieBundle, VelloLottieHandle,
    };
    #[cfg(feature = "svg")]
    pub use crate::integrations::svg::{VelloSvg, VelloSvgAnchor, VelloSvgBundle, VelloSvgHandle};
    #[cfg(feature = "text")]
    pub use crate::integrations::text::{
        VelloFont, VelloTextAlign, VelloTextAnchor, VelloTextBundle, VelloTextSection,
        VelloTextStyle,
    };
    pub use crate::{
        VelloScene, VelloSceneBundle,
        render::{SkipEncoding, VelloRenderSettings, VelloView},
    };
}

#[derive(Bundle, Default)]
pub struct VelloSceneBundle {
    /// Scene to render
    pub scene: VelloScene,
    /// A transform to apply to this scene
    pub transform: Transform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
    /// A bucket into which we group entities for the purposes of visibility.
    pub visibility_class: VisibilityClass,
}

/// A simple newtype component wrapper for [`vello::Scene`] for rendering.
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(Transform, Visibility, VisibilityClass)]
#[component(on_add = view::add_visibility_class::<VelloScene>)]
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
