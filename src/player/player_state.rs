use super::PlayerTransition;
use crate::{PlaybackOptions, Theme, VelloAsset};
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: &'static str,
    pub asset: Option<Handle<VelloAsset>>,
    pub theme: Theme,
    pub options: PlaybackOptions,
    pub transitions: Vec<PlayerTransition>,
    /// Whether to reset the playhead when you transition away from this state
    pub reset_playhead_on_transition: bool,
    /// Whether to reset the playhead when the transition it moved to this
    /// state
    pub reset_playhead_on_start: bool,
}

impl PlayerState {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            asset: Default::default(),
            options: Default::default(),
            theme: Default::default(),
            transitions: vec![],
            reset_playhead_on_transition: false,
            reset_playhead_on_start: false,
        }
    }

    pub fn with_asset(mut self, asset: Handle<VelloAsset>) -> Self {
        self.asset.replace(asset);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_playback_options(mut self, options: PlaybackOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_transition(mut self, transition: PlayerTransition) -> Self {
        self.transitions.push(transition);
        self
    }

    pub fn reset_playhead_on_transition(mut self, reset: bool) -> Self {
        self.reset_playhead_on_transition = reset;
        self
    }

    pub fn reset_playhead_on_start(mut self, reset: bool) -> Self {
        self.reset_playhead_on_start = reset;
        self
    }
}
