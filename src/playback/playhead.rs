use bevy::{prelude::*, utils::Instant};

#[derive(PartialEq, Component, Clone, Debug)]
pub struct Playhead {
    pub(crate) first_render: Option<Instant>,
    pub(crate) frame: f32,
    pub(crate) intermission: Option<Timer>,
    pub(crate) loops_completed: usize,
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

    /// Reset the amount of loops completed
    pub fn reset_loops(&mut self) {
        self.loops_completed = 0;
    }

    pub(crate) fn new(frame: f32) -> Self {
        Self {
            frame,
            first_render: None,
            intermission: None,
            loops_completed: 0,
        }
    }
}
