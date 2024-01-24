use crate::{metadata::Metadata, Origin};
use bevy::{
    math::{Vec3A, Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    reflect::TypePath,
    utils::Instant,
};
use std::sync::Arc;
use vello::SceneFragment;

#[derive(Clone)]
pub enum Vector {
    Svg {
        /// The original image encoding
        original: Arc<SceneFragment>,
        /// The time we started rendering this asset
        first_frame: Option<Instant>,
    },
    Lottie {
        /// The original image encoding
        composition: Arc<vellottie::Composition>,
        /// The time we started rendering this asset
        first_frame: Option<Instant>,
        /// The last frame rendered
        rendered_frames: f32,
    },
}

#[derive(Asset, TypePath, Clone)]
pub struct VelloAsset {
    pub data: Vector,
    pub local_transform_bottom_center: Transform,
    pub local_transform_center: Transform,
    pub width: f32,
    pub height: f32,
}

impl VelloAsset {
    pub fn center_in_world(&self, transform: &GlobalTransform, origin: &Origin) -> Vec2 {
        let world_transform = transform.compute_matrix();
        let local_transform = match origin {
            Origin::BottomCenter => self
                .local_transform_bottom_center
                .compute_matrix()
                .inverse(),
            Origin::Center => return transform.translation().xy(),
        };

        let local_center_point = Vec3A::new(self.width / 2.0, -self.height / 2.0, 0.0);

        (world_transform * local_transform * local_center_point.extend(1.0)).xy()
    }

    /// Returns the 4 corner points of this vector's bounding box in world space
    pub fn bb_in_world_space(&self, transform: &GlobalTransform, origin: &Origin) -> [Vec2; 4] {
        let min = Vec3A::ZERO;
        let x_axis = Vec3A::new(self.width, 0.0, 0.0);

        let max = Vec3A::new(self.width, -self.height, 0.0);
        let y_axis = Vec3A::new(0.0, -self.height, 0.0);

        let world_transform = transform.compute_matrix();
        let local_transform = match origin {
            Origin::BottomCenter => self
                .local_transform_bottom_center
                .compute_matrix()
                .inverse(),
            Origin::Center => self.local_transform_center.compute_matrix().inverse(),
        };

        let min = (world_transform * local_transform * min.extend(1.0)).xy();
        let x_axis = (world_transform * local_transform * x_axis.extend(1.0)).xy();
        let max = (world_transform * local_transform * max.extend(1.0)).xy();
        let y_axis = (world_transform * local_transform * y_axis.extend(1.0)).xy();

        [min, x_axis, max, y_axis]
    }

    pub fn bb_in_screen_space(&self, transform: &GlobalTransform) -> [Vec2; 4] {
        let min = Vec3A::ZERO;
        let x_axis = Vec3A::new(self.width, 0.0, 0.0);

        let max = Vec3A::new(self.width, -self.height, 0.0);
        let y_axis = Vec3A::new(0.0, -self.height, 0.0);

        let world_transform = transform.compute_matrix();
        let local_transform = self.local_transform_center.compute_matrix().inverse();

        let min = (world_transform * local_transform * min.extend(1.0)).xy();
        let x_axis = (world_transform * local_transform * x_axis.extend(1.0)).xy();
        let max = (world_transform * local_transform * max.extend(1.0)).xy();
        let y_axis = (world_transform * local_transform * y_axis.extend(1.0)).xy();

        [min, x_axis, max, y_axis]
    }

    /// Gets the lottie metadata (if vector is a lottie), an object used for inspecting
    /// this vector's layers and shapes
    pub fn metadata(&self) -> Option<Metadata> {
        if let Vector::Lottie { composition, .. } = &self.data {
            Some(Metadata {
                composition: composition.clone(),
            })
        } else {
            None
        }
    }
}
