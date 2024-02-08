use bevy::prelude::*;
use std::ops::Range;

#[derive(PartialEq, Component, Clone, Debug, Reflect)]
#[reflect(Component)]
/// Playback settings which adjust the playback of a vello asset.
///
/// You can add this component directly to a `VelloAssetBundle` entity to adjust playback settings.
pub struct PlaybackSettings {
    pub autoplay: bool,
    pub direction: PlaybackDirection,
    pub speed: f32,
    pub intermission: f32,
    pub looping: PlaybackLoopBehavior,
    pub segments: Range<f32>,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            autoplay: true,
            direction: PlaybackDirection::default(),
            speed: 1.0,
            intermission: 0.0,
            looping: PlaybackLoopBehavior::default(),
            segments: f32::MIN..f32::MAX,
        }
    }
}

/// The direction to play the segments of a lottie animation.
#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum PlaybackDirection {
    #[default]
    Normal = 1,
    Reverse = -1,
}

/// How often to loop.
#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum PlaybackLoopBehavior {
    None,
    Amount(usize),
    #[default]
    Loop,
}
