#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] -- This would be great! But we are far away.
//! An integration to render SVG and Lottie assets in Bevy with Vello.

mod plugin;
pub use plugin::VelloPlugin;

#[cfg(feature = "picking")]
mod picking;

pub mod integrations;
pub mod render;

// Re-exports
pub use imaging;
pub use imaging_vello;
#[cfg(feature = "text")]
pub use parley;
#[cfg(feature = "svg")]
pub use svg_imaging;
#[cfg(feature = "lottie")]
pub use velato;
#[cfg(feature = "lottie")]
pub use velato_imaging;
pub use vello;

pub mod prelude {
    // Vendor re-exports
    pub use vello::{self, kurbo, peniko};

    pub use crate::{
        integrations::scene::{UiVelloScene, VelloScene2d},
        render::{VelloRenderSettings, VelloView},
    };

    #[cfg(feature = "lottie")]
    pub use crate::integrations::lottie::{
        LottieExt, LottiePlayer, PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions,
        PlaybackPlayMode, PlayerState, PlayerTransition, Playhead, Theme, UiVelloLottie,
        VelloLottie, VelloLottie2d, VelloLottieAnchor,
    };
    #[cfg(feature = "svg")]
    pub use crate::integrations::svg::{UiVelloSvg, VelloSvg, VelloSvg2d, VelloSvgAnchor};
    #[cfg(feature = "text")]
    pub use crate::integrations::text::{
        UiVelloText, VelloFont, VelloText2d, VelloTextAlign, VelloTextAnchor, VelloTextStyle,
    };
}
