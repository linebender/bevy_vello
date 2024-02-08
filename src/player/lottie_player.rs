use super::PlayerState;
use bevy::{prelude::*, utils::hashbrown::HashMap};

/// A lottie player that closely mirrors the behavior and functionality for dotLottie Interactivity.
///
/// See: https://docs.lottiefiles.com/dotlottie-js-external/
#[derive(Component, Clone, Default, Debug)]
pub struct LottiePlayer {
    initial_state: &'static str,
    current_state: &'static str,
    next_state: Option<&'static str>,
    states: HashMap<&'static str, PlayerState>,
    /// Whether the player has started.
    started: bool,
    /// Whether the player is playing. State machines will continue unless stopped.
    playing: bool,
    /// Stopped. Doesn't run state machines.
    stopped: bool,
}

impl LottiePlayer {
    /// Retrieve an immutable reference to the current state.
    pub fn state(&self) -> &PlayerState {
        self.states
            .get(self.current_state)
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state))
    }

    /// Retrieve a mutable reference to the current state.
    pub fn state_mut(&mut self) -> &mut PlayerState {
        self.states
            .get_mut(self.current_state)
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state))
    }

    /// Returns an immutable iterator of the states for this player.
    pub fn states(&self) -> impl Iterator<Item = &PlayerState> {
        self.states.values()
    }

    /// Returns a mutable iterator of the states for this player.
    pub fn states_mut(&mut self) -> impl Iterator<Item = &mut PlayerState> {
        self.states.values_mut()
    }

    /// Transition to the named state.
    pub fn transition(&mut self, state: &'static str) {
        self.next_state.replace(state);
    }

    /// Resets or goes back to the default/initial animation.
    pub fn reset(&mut self) {
        self.next_state = Some(self.initial_state);
        self.seek(f32::MIN);
    }

    /// The playhead (frame) last rendered
    pub fn playhead(&self) -> f32 {
        self.playhead
    }

    /// Seeks to a specific frame.
    pub fn seek(&mut self, frame: f32) {
        self.playhead = frame;
    }

    /// Toggle the play state.
    pub fn toggle_play(&mut self) {
        if self.stopped || !self.playing {
            self.play();
        } else {
            self.pause();
        }
    }

    /// Play the animation.
    pub fn play(&mut self) {
        self.playing = true;
        self.stopped = false;
    }

    /// Pauses the animation. State machines will continue.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stops the animation. State machines will not run.
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }
}

impl LottiePlayer {
    pub fn new(initial_state: &'static str) -> LottiePlayer {
        LottiePlayer {
            initial_state,
            current_state: initial_state,
            next_state: Some(initial_state),
            states: HashMap::new(),
            started: false,
            playing: false,
            stopped: false,
        }
    }

    pub fn with_state(mut self, state: PlayerState) -> Self {
        self.states.insert(state.id, state);
        self
    }
}
