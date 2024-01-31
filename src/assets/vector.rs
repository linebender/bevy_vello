use crate::{metadata::Metadata, AnimationDirection, Origin, PlaybackSettings};
use bevy::{
    math::{Vec3A, Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    reflect::TypePath,
    utils::Instant,
};
use std::sync::Arc;
use vello::SceneFragment;
use vello_svg::usvg::strict_num::Ulps;

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

    /// Calculate the playhead. Returns `None` is the Vector is an SVG.
    pub fn calculate_playhead(&self, playback_settings: &PlaybackSettings) -> Option<f32> {
        let Vector::Lottie {
            composition,
            first_frame: _,
            rendered_frames,
        } = &self.data
        else {
            return None;
        };

        let start_frame = playback_settings
            .segments
            .start
            .max(composition.frames.start);
        let end_frame = playback_settings.segments.end.min(composition.frames.end);
        let length = end_frame - start_frame + playback_settings.intermission;

        let loop_frame = match playback_settings.looping {
            crate::AnimationLoopBehavior::None => rendered_frames.min(length.prev()),
            crate::AnimationLoopBehavior::Amount(loops) => {
                rendered_frames.min((loops as f32 * length).prev())
            }
            crate::AnimationLoopBehavior::Loop => *rendered_frames,
        };
        // Normalize frame
        let normal_frame = loop_frame % length;
        debug!("loop frame: {loop_frame}, normal = {normal_frame}");
        let playhead = match playback_settings.direction {
            AnimationDirection::Normal => start_frame + normal_frame,
            AnimationDirection::Reverse => end_frame - normal_frame,
        }
        .clamp(start_frame, end_frame.prev());
        Some(playhead)
    }
}
