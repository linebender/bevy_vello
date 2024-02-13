use bevy::{math::Vec3A, prelude::*};
use vello::peniko::Brush;

use crate::VelloFont;

#[derive(Component, Default, Clone)]
pub struct VelloText {
    pub content: String,
    pub size: f32,
    pub brush: Option<Brush>,
}

impl VelloText {
    /// Returns the position and size relative to the given transform's space (world or screen)
    pub fn bb_in_space(
        &self,
        font: &VelloFont,
        transform: &Transform,
        gtransform: &GlobalTransform,
    ) -> Rect {
        let font_bb = font.sizeof(self);

        let min = Vec3A::ZERO;

        let max = Vec3A::new(font_bb.x, font_bb.y, 0.0);

        let world_transform = gtransform.compute_matrix();
        let local_transform = transform.compute_matrix().inverse();

        let min = (world_transform * local_transform * min.extend(1.0)).xy();
        let max = (world_transform * local_transform * max.extend(1.0)).xy();

        Rect { min, max }
    }
}
