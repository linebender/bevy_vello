mod asset_loader;

pub(crate) mod render;

pub mod asset;
pub use asset::VelloLottie;

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
pub use theme::Theme;

/// A renderable Lottie in the world.
use bevy::{
    camera::{primitives::Aabb, visibility::VisibilityClass},
    prelude::*,
};
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(
    Aabb,
    VelloLottieAnchor,
    Playhead,
    PlaybackOptions,
    LottiePlayer,
    Transform,
    Visibility,
    VisibilityClass
)]
#[reflect(Component)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<VelloLottie2d>)]
pub struct VelloLottie2d(pub Handle<VelloLottie>);

/// A renderable Lottie that may be used in Bevy UI.
///
/// ### Object fit
/// The image will preserve the aspect ratio, and fits the image inside the container, without cutting - will leave empty space if needed.
#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq, Reflect)]
#[require(
    Node,
    VelloLottieAnchor,
    Playhead,
    PlaybackOptions,
    LottiePlayer,
    UiTransform,
    Visibility,
    VisibilityClass
)]
#[reflect(Component)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<UiVelloLottie>)]
pub struct UiVelloLottie(pub Handle<VelloLottie>);

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
