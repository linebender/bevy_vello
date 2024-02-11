use bevy::{prelude::*, utils::Instant};

#[derive(PartialEq, Component, Clone, Debug)]
pub struct Playhead {
    /// Used to track transitions relating to time.
    pub(crate) first_render: Option<Instant>,
    /// The actual frame being rendered
    pub(crate) frame: f32,
    /// Used to track intermission.
    pub(crate) intermission: Option<Timer>,
    /// Used to count loops for loop behavior.
    pub(crate) loops_completed: usize,
    /// Used by play mode to track current direction. Only set to -1.0
    /// (reverse) or 1.0 (normal).
    pub(crate) playmode_dir: f32,
}

impl Playhead {
    /// Get the current playhead frame
    pub fn frame(&self) -> f32 {
        self.frame
    }

    /// Seek to a given frame
    pub fn seek(&mut self, frame: f32) {
        self.frame = frame;
    }

    pub(crate) fn new(frame: f32) -> Self {
        Self {
            frame,
            first_render: None,
            intermission: None,
            loops_completed: 0,
            playmode_dir: 1.0,
        }
    }
}
