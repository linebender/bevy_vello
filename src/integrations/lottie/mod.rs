mod asset_loader;

pub(crate) mod render;

pub mod asset;
pub use asset::{VelloLottie, VelloLottieHandle};

mod systems;

mod parse;
pub use parse::{load_lottie_from_bytes, load_lottie_from_str};

mod lottie_ext;
pub use lottie_ext::LottieExt;

mod plugin;
pub(crate) use plugin::LottieIntegrationPlugin;

mod playback_options;
pub use playback_options::{
    PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions, PlaybackPlayMode,
};

mod playhead;
pub use playhead::Playhead;

mod lottie_player;
pub use lottie_player::LottiePlayer;

mod player_state;
pub use player_state::PlayerState;

mod player_transition;
pub use player_transition::PlayerTransition;

mod theme;
use bevy::{camera::visibility::VisibilityClass, prelude::*};
pub use theme::Theme;

#[cfg(feature = "lottie")]
#[derive(Bundle, Default)]
pub struct VelloLottieBundle {
    /// Asset data to render
    pub asset: VelloLottieHandle,
    /// How the asset is positioned relative to its [`Transform`].
    pub asset_anchor: VelloLottieAnchor,
    /// The current playhead for the animation
    pub playhead: Playhead,
    /// The playback options for the animation
    pub playback_options: PlaybackOptions,
    /// The player used for advanced state machine transitions and playback control.
    pub player: LottiePlayer,
    /// A transform to apply to this vector
    pub transform: Transform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
    /// A bucket into which we group entities for the purposes of visibility.
    pub visibility_class: VisibilityClass,
}

/// Describes how the asset is positioned relative to its [`Transform`]. It defaults to
/// [`VelloLottieAnchor::Center`].
///
/// Has no effect in UI nodes.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum VelloLottieAnchor {
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
