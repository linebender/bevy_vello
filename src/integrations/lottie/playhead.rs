use bevy::prelude::*;
use bevy::utils::Instant;

/// The playhead for a vello asset. This cannot be constructed by the user, it is created automatically and available on the first frame.
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

impl Playhead {
    /// Get the current playhead frame
    pub fn frame(&self) -> f64 {
        self.frame
    }

    /// Seek to a given frame
    pub fn seek(&mut self, frame: f64) {
        self.frame = frame;
    }

    pub(crate) fn new(frame: f64) -> Self {
        Self {
            frame,
            first_render: None,
            intermission: None,
            loops_completed: 0,
            playmode_dir: 1.0,
        }
    }
}
