use crate::integrations::lottie::LottieAssetVariant;
use bevy::platform::collections::HashMap;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use std::{ops::Range, time::Duration};

pub(super) mod events;
pub(super) mod hooks;

mod state;
pub use state::PlayerState;

/// A lottie player that allows runtime manipulation of Lottie animations.
///
/// Controls lottie playback and transitions with state machine support.
#[derive(Component, Clone, Debug)]
pub struct LottiePlayer<A: LottieAssetVariant> {
    /// Whether the player has started.
    pub(crate) started: bool,
    /// Whether the player is playing. State machines will continue unless
    /// stopped.
    pub(crate) playing: bool,
    /// Stopped. Doesn't run state machines.
    pub(crate) stopped: bool,
    pub(crate) current_state: Option<&'static str>,
    pub(crate) next_state: Option<&'static str>,
    pub(crate) states: HashMap<&'static str, PlayerState<A>>,
}

impl<A: LottieAssetVariant> Default for LottiePlayer<A> {
    fn default() -> Self {
        let mut states = HashMap::new();
        states.insert("default", PlayerState::<A>::new("default"));
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

impl<A: LottieAssetVariant> LottiePlayer<A> {
    pub fn new(initial_state: &'static str) -> LottiePlayer<A> {
        LottiePlayer::<A> {
            current_state: None,
            next_state: Some(initial_state),
            states: HashMap::new(),
            started: false,
            playing: false,
            stopped: false,
        }
    }

    pub fn with_state(mut self, state: PlayerState<A>) -> Self {
        self.states.insert(state.id, state);
        self
    }

    /// Retrieve an immutable reference to the current state.
    pub fn state(&self) -> &PlayerState<A> {
        self.states
            .get(
                self.current_state
                    .or(self.next_state)
                    .expect("expected state"),
            )
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state.unwrap()))
    }

    /// Retrieve a mutable reference to the current state.
    pub fn state_mut(&mut self) -> &mut PlayerState<A> {
        self.states
            .get_mut(
                self.current_state
                    .or(self.next_state)
                    .expect("expected state"),
            )
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state.unwrap()))
    }

    /// Returns an immutable iterator of the states for this player.
    pub fn states(&self) -> impl Iterator<Item = &PlayerState<A>> {
        self.states.values()
    }

    /// Returns a mutable iterator of the states for this player.
    pub fn states_mut(&mut self) -> impl Iterator<Item = &mut PlayerState<A>> {
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
#[reflect(Component)]
pub enum PlaybackDirection {
    /// Play in the default direction, first frame to last frame.
    #[default]
    Normal = 1,
    /// Play in the reverse direction, last frame to first frame.
    Reverse = -1,
}

/// How often to loop.
#[derive(PartialEq, Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
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
#[reflect(Component)]
pub enum PlaybackPlayMode {
    /// Reset the playhead every loop.
    #[default]
    Normal,
    /// Reverse the direction every loop.
    Bounce,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum PlayerTransition {
    /// Transitions to the given state after a period of seconds.
    OnAfter { state: &'static str, secs: f32 },
    /// Transition to the given state after the animation finishes.
    OnComplete { state: &'static str },
    /// Transition to the given state when the mouse enters the image bounding box.
    OnMouseEnter { state: &'static str },
    /// Transition to the given state when the mouse clicks inside the image bounding box.
    OnMouseClick { state: &'static str },
    /// Transition to the given state when the mouse exits the image bounding box.
    OnMouseLeave { state: &'static str },
    /// Transition to the given state on first render of this state.
    OnShow { state: &'static str },
}

/// The playhead for a lottie.
#[derive(PartialEq, Component, Clone, Debug)]
pub struct Playhead {
    /// Used to track transitions relating to time.
    pub(crate) first_render: Option<Instant>,
    /// The actual frame being rendered
    pub(crate) frame: f64,
    /// Used to track intermission.
    pub(crate) intermission: Option<Timer>,
    /// Used to count loops for loop behavior.
    pub(crate) loops_completed: usize,
    /// Used by play mode to track current direction. Only set to -1.0
    /// (reverse) or 1.0 (normal).
    pub(crate) playmode_dir: f64,
}

impl Default for Playhead {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Playhead {
    /// Get the current playhead frame
    pub fn frame(&self) -> f64 {
        self.frame
    }

    /// Seek to a given frame
    pub fn seek(&mut self, frame: f64) {
        self.frame = frame;
    }

    pub fn new(frame: f64) -> Self {
        Self {
            frame,
            first_render: None,
            intermission: None,
            loops_completed: 0,
            playmode_dir: 1.0,
        }
    }
}
