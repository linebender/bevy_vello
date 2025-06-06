use bevy::{platform::collections::HashMap, prelude::*};

use super::PlayerState;

/// A lottie player that allows runtime manipulation of Lottie animations.
///
/// Controls lottie playback and transitions with state machine support.
#[derive(Component, Clone, Debug)]
pub struct LottiePlayer {
    pub(crate) current_state: Option<&'static str>,
    pub(crate) next_state: Option<&'static str>,
    pub(crate) states: HashMap<&'static str, PlayerState>,
    /// Whether the player has started.
    pub(crate) started: bool,
    /// Whether the player is playing. State machines will continue unless
    /// stopped.
    pub(crate) playing: bool,
    /// Stopped. Doesn't run state machines.
    pub(crate) stopped: bool,
}

impl Default for LottiePlayer {
    fn default() -> Self {
        let mut states = HashMap::new();
        states.insert("default", PlayerState::new("default"));
        Self {
            current_state: Some("default"),
            next_state: None,
            states,
            started: false,
            playing: true,
            stopped: false,
        }
    }
}

impl LottiePlayer {
    /// Retrieve an immutable reference to the current state.
    pub fn state(&self) -> &PlayerState {
        self.states
            .get(
                self.current_state
                    .or(self.next_state)
                    .expect("expected state"),
            )
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state.unwrap()))
    }

    /// Retrieve a mutable reference to the current state.
    pub fn state_mut(&mut self) -> &mut PlayerState {
        self.states
            .get_mut(
                self.current_state
                    .or(self.next_state)
                    .expect("expected state"),
            )
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state.unwrap()))
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
            current_state: None,
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
