use super::Metadata;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use std::sync::Arc;

#[derive(Clone)]
pub enum VectorFile {
    Svg(Arc<vello::Scene>),
    Lottie(Arc<velato::Composition>),
}

#[derive(Asset, TypePath, Clone)]
pub struct VelloAsset {
    pub data: VectorFile,
    pub local_transform_center: Transform,
    pub width: f32,
    pub height: f32,
}

impl VelloAsset {
    /// Returns the bounding box in world space
    pub fn bb_in_world_space(&self, gtransform: &GlobalTransform) -> Rect {
        // Convert local coordinates to world coordinates
        let local_min = Vec3::new(-self.width / 2.0, -self.height / 2.0, 0.0).extend(1.0);
        let local_max = Vec3::new(self.width / 2.0, self.height / 2.0, 0.0).extend(1.0);

        let min_world = gtransform.compute_matrix() * local_min;
        let max_world = gtransform.compute_matrix() * local_max;

        // Calculate the distance between the vertices to get the size in world space
        let min = Vec2::new(min_world.x, min_world.y);
        let max = Vec2::new(max_world.x, max_world.y);
        Rect { min, max }
    }

    /// Returns the bounding box in space space
    pub fn bb_in_screen_space(
        &self,
        gtransform: &GlobalTransform,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Rect> {
        let Rect { min, max } = self.bb_in_world_space(gtransform);
        camera
            .viewport_to_world_2d(camera_transform, min)
            .zip(camera.viewport_to_world_2d(camera_transform, max))
            .map(|(min, max)| Rect { min, max })
    }

    /// Gets the lottie metadata (if vector is a lottie), an object used for
    /// inspecting this vector's layers and shapes
    pub fn metadata(&self) -> Option<Metadata> {
        if let VectorFile::Lottie(composition) = &self.data {
            Some(Metadata {
                composition: composition.clone(),
            })
        } else {
            None
        }
    }
}

/// Describes how to position the asset from the origin
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum VelloAssetAlignment {
    /// Bounds start from the render position and advance up and to the right.
    BottomLeft,
    /// Bounds start from the render position and advance up.
    Bottom,
    /// Bounds start from the render position and advance up and to the left.
    BottomRight,

    /// Bounds start from the render position and advance right.
    Left,
    /// Bounds start from the render position and advance equally on both axes.
    #[default]
    Center,
    /// Bounds start from the render position and advance left.
    Right,

    /// Bounds start from the render position and advance down and to the right.
    TopLeft,
    /// Bounds start from the render position and advance down.
    Top,
    /// Bounds start from the render position and advance down and to the left.
    TopRight,
}
