use super::PlayerTransition;
use crate::{PlaybackSettings, Theme, VelloAsset};
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: &'static str,
    pub asset: Option<Handle<VelloAsset>>,
    pub theme: Option<Theme>,
    pub playback_settings: Option<PlaybackSettings>,
    pub transitions: Vec<PlayerTransition>,
    /// Whether to reset the playhead when you transition away from this state
    pub reset_playhead_on_transition: bool,
    /// Whether to reset the playhead when the transition it moved to this state
    pub reset_playhead_on_start: bool,
}

impl PlayerState {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            asset: Default::default(),
            playback_settings: None,
            theme: None,
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
        self.theme.replace(theme);
        self
    }

    pub fn with_playback_settings(mut self, playback_settings: PlaybackSettings) -> Self {
        self.playback_settings.replace(playback_settings);
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
