use std::sync::Arc;

use bevy::{prelude::*, reflect::TypePath};

use crate::prelude::*;

#[derive(Asset, TypePath, Clone)]
pub struct VelloSvg {
    pub scene: Arc<vello::Scene>,
    pub width: f32,
    pub height: f32,
    pub alpha: f32,
}

impl VelloSvg {
    /// Returns the bounding box in world space
    pub fn bb_in_world_space(&self, gtransform: &GlobalTransform) -> Rect {
        // Convert local coordinates to world coordinates
        let local_min = Vec3::new(-self.width / 2.0, -self.height / 2.0, 0.0).extend(1.0);
        let local_max = Vec3::new(self.width / 2.0, self.height / 2.0, 0.0).extend(1.0);

        let min_world = gtransform.to_matrix() * local_min;
        let max_world = gtransform.to_matrix() * local_max;

        // Calculate the distance between the vertices to get the size in world space
        let min = Vec2::new(min_world.x, min_world.y);
        let max = Vec2::new(max_world.x, max_world.y);
        Rect { min, max }
    }

    /// Returns the bounding box in screen space
    pub fn bb_in_screen_space(
        &self,
        gtransform: &GlobalTransform,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Rect> {
        let Rect { min, max } = self.bb_in_world_space(gtransform);
        camera
            .viewport_to_world_2d(camera_transform, min)
            .ok()
            .zip(camera.viewport_to_world_2d(camera_transform, max).ok())
            .map(|(min, max)| Rect { min, max })
    }
}
