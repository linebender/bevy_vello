use bevy::{prelude::*, utils::Instant};

#[derive(PartialEq, Component, Clone, Debug)]
pub struct Playhead {
    first_render: Option<Instant>,
    frame: f32,
    intermission_frame: f32,
    loops_completed: usize,
}

impl Playhead {
    /// Get the current playhead
    pub fn playhead(&self) -> f32 {
        self.frame
    }

    /// Seek to a frame
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

    pub(crate) fn reset_to(&mut self, frame: f32) {
        self.frame = frame;
        self.intermission_frame = 0.0;
        self.loops_completed = 0;
        self.first_render.take();
    }
}
