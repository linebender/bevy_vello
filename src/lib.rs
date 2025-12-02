#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] -- This would be great! But we are far away.
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use bevy::prelude::*;

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
    // Vendor re-exports
    pub use vello::{self, kurbo, peniko};

    pub use crate::{
        VelloRenderSpace,
        integrations::scene::{VelloScene, VelloSceneBundle},
        render::{
            SkipEncoding, SkipScaling, VelloRenderSettings, VelloScreenScale, VelloView,
            VelloWorldScale,
        },
    };

    #[cfg(feature = "lottie")]
    pub use crate::integrations::lottie::{
        LottieExt, LottiePlayer, PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions,
        PlaybackPlayMode, PlayerState, PlayerTransition, Playhead, Theme, VelloLottie,
        VelloLottieAnchor, VelloLottieBundle, VelloLottieHandle,
    };
    #[cfg(feature = "svg")]
    pub use crate::integrations::svg::{UiVelloSvg, VelloSvg, VelloSvg2d, VelloSvgAnchor};
    #[cfg(feature = "text")]
    pub use crate::integrations::text::{
        VelloFont, VelloTextAlign, VelloTextAnchor, VelloTextBundle, VelloTextSection,
        VelloTextStyle,
    };
}

/// Determines which coordinate space this entity should be rendered in.
///
/// - `VelloRenderSpace::World`
///   Renders using **world-space coordinates**, typically with an origin at the
///   center of the scene and a **Y-up** coordinate system. World-space rendering
///   is affected by the world camera's transform.
///
/// - `VelloRenderSpace::Screen`
///   Renders using **screen-space coordinates**, typically with an origin at the
///   top-left of the window and a **Y-down** coordinate system. Screen-space
///   rendering is anchored to the screen and is **not affected** by world camera
///   movement.
///
/// If this component is attached to an entity that is parented under a Bevy UI
/// `Node`, it is ignored; the entity will render in **UI layout space** instead.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum VelloRenderSpace {
    #[default]
    World,
    Screen,
}
