use bevy::prelude::*;
use std::ops::Range;

#[derive(PartialEq, Component, Clone, Debug, Reflect)]
#[reflect(Component)]
/// Add this component to a `VelloAssetBundle` entity to adjust playback settings.
pub struct PlaybackSettings {
    pub autoplay: bool,
    pub direction: PlaybackDirection,
    pub speed: f32,
    pub looping: bool,
    pub segments: Range<f32>,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            autoplay: false,
            direction: PlaybackDirection::default(),
            speed: 1.0,
            looping: true,
            segments: f32::MIN..f32::MAX,
        }
    }
}

#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum PlaybackDirection {
    #[default]
    Normal = 1,
    Reverse = -1,
}
