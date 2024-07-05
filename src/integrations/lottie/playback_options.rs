//! Playback options for lottie files.

use bevy::prelude::*;
use std::{ops::Range, time::Duration};

/// Playback options which adjust the playback of an asset.
///
/// You can add this component directly to a `VelloAssetBundle` entity to adjust
/// playback options.
#[derive(PartialEq, Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct PlaybackOptions {
    /// Whether to automatically start the animation.
    pub autoplay: bool,
    /// The direction of the animation.
    pub direction: PlaybackDirection,
    /// The speed of the animation as a multiplier. 1.0 is normal speed.
    /// Anything less than 1 is slower, and anything greater than 1 is faster.
    pub speed: f64,
    /// A duration of time spent idle between loops.
    pub intermission: Duration,
    /// Whether to reset the playhead every loop (normal) or to reverse
    /// directions (bounce).
    pub play_mode: PlaybackPlayMode,
    /// Whether to loop, and how many.
    pub looping: PlaybackLoopBehavior,
    /// The segments (frames) of the animation to play. Values out of range
    /// will be ignored.
    pub segments: Range<f64>,
}

impl Default for PlaybackOptions {
    fn default() -> Self {
        Self {
            autoplay: true,
            direction: Default::default(),
            speed: 1.0,
            intermission: Duration::ZERO,
            play_mode: Default::default(),
            looping: Default::default(),
            segments: f64::MIN..f64::MAX,
        }
    }
}

/// The direction to play the segments of a lottie animation.
#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum PlaybackDirection {
    /// Play in the default direction, first frame to last frame.
    #[default]
    Normal = 1,
    /// Play in the reverse direction, last frame to first frame.
    Reverse = -1,
}

/// How often to loop.
#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum PlaybackLoopBehavior {
    /// Do not loop. This is equivalent to `PlaybackLoopBehavior::Amount(0)`.
    DoNotLoop,
    /// Complete a specified number of loops.
    Amount(usize),
    /// Loop continuously.
    #[default]
    Loop,
}

/// Whether to reset (normal) the playhead every loop or to reverse directions
/// (bounce).
#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum PlaybackPlayMode {
    /// Reset the playhead every loop.
    #[default]
    Normal,
    /// Reverse the direction every loop.
    Bounce,
}
