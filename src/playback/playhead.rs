use bevy::{prelude::*, utils::Instant};

#[derive(PartialEq, Component, Clone, Debug)]
pub struct Playhead {
    pub(crate) first_render: Option<Instant>,
    pub(crate) frame: f32,
    pub(crate) intermission_frame: f32,
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

    pub(crate) fn new(frame: f32) -> Self {
        Self {
            frame,
            first_render: None,
            intermission_frame: 0.0,
            loops_completed: 0,
        }
    }
}
