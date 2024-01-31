use bevy::prelude::*;
use std::ops::Range;

#[derive(PartialEq, Component, Clone, Debug, Reflect)]
#[reflect(Component)]
/// Add this component to a `VelloAssetBundle` entity to adjust playback settings.
pub struct PlaybackSettings {
    pub autoplay: bool,
    pub direction: AnimationDirection,
    pub speed: f32,
    pub intermission: f32,
    pub looping: AnimationLoopBehavior,
    pub play_mode: AnimationPlayMode,
    pub segments: Range<f32>,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            autoplay: true,
            direction: AnimationDirection::default(),
            speed: 1.0,
            intermission: 0.0,
            looping: AnimationLoopBehavior::default(),
            play_mode: AnimationPlayMode::default(),
            segments: f32::MIN..f32::MAX,
        }
    }
}

#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum AnimationDirection {
    #[default]
    Normal = 1,
    Reverse = -1,
}

#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum AnimationLoopBehavior {
    None,
    Amount(usize),
    #[default]
    Loop,
}

#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
pub enum AnimationPlayMode {
    #[default]
    Normal,
    Bounce,
}
