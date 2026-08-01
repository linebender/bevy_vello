use bevy::{camera::primitives::Aabb, prelude::*};

/// Describes how the asset is positioned relative to its [`Transform`]. It defaults to
/// [`VelloAnchor::Center`].
///
/// Has no effect in UI nodes and scenes.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub enum VelloAnchor {
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
    /// Custom anchor offset from [`VelloAnchor::Center`] in local Bevy space.
    CenterOffset(Vec2),
    /// Bounds start from the render position and advance left.
    Right,

    /// Bounds start from the render position and advance down and to the right.
    TopLeft,
    /// Bounds start from the render position and advance down.
    Top,
    /// Bounds start from the render position and advance down and to the left.
    TopRight,
}

impl VelloAnchor {
    pub fn to_local_from_dimensions(&self, width: f32, height: f32) -> Vec3 {
        match self {
            VelloAnchor::TopLeft => Vec3::ZERO,
            VelloAnchor::Left => Vec3::new(0.0, height / 2.0, 0.0),
            VelloAnchor::BottomLeft => Vec3::new(0.0, height, 0.0),
            VelloAnchor::Top => Vec3::new(width / 2.0, 0.0, 0.0),
            VelloAnchor::Center => Vec3::new(width / 2.0, height / 2.0, 0.0),
            VelloAnchor::Bottom => Vec3::new(width / 2.0, height, 0.0),
            VelloAnchor::TopRight => Vec3::new(width, 0.0, 0.0),
            VelloAnchor::Right => Vec3::new(width, height / 2.0, 0.0),
            VelloAnchor::BottomRight => Vec3::new(width, height, 0.0),
            // Custom offsets are expressed from the centered anchor in Bevy's local space.
            // Vello's local asset coordinates use a top-left origin with +Y pointing down.
            VelloAnchor::CenterOffset(offset) => {
                Vec3::new(width / 2.0 - offset.x, height / 2.0 + offset.y, 0.0)
            }
        }
    }

    pub fn to_aabb_from_dimensions(&self, width: f32, height: f32) -> Aabb {
        let half_size = Vec3::new(width / 2.0, height / 2.0, 0.0);
        let (dx, dy) = {
            match self {
                VelloAnchor::TopLeft => (half_size.x, -half_size.y),
                VelloAnchor::Left => (half_size.x, 0.0),
                VelloAnchor::BottomLeft => (half_size.x, half_size.y),
                VelloAnchor::Top => (0.0, -half_size.y),
                VelloAnchor::Center => (0.0, 0.0),
                VelloAnchor::Bottom => (0.0, half_size.y),
                VelloAnchor::TopRight => (-half_size.x, -half_size.y),
                VelloAnchor::Right => (-half_size.x, 0.0),
                VelloAnchor::BottomRight => (-half_size.x, half_size.y),
                VelloAnchor::CenterOffset(offset) => (offset.x, offset.y),
            }
        };
        let adjustment = Vec3::new(dx, dy, 0.0);
        let min = -half_size + adjustment;
        let max = half_size + adjustment;
        Aabb::from_min_max(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_local_anchor_offsets_from_center() {
        let anchor = VelloAnchor::CenterOffset(Vec2::new(25.0, 10.0));

        assert_eq!(
            anchor.to_local_from_dimensions(200.0, 100.0),
            Vec3::new(75.0, 60.0, 0.0)
        );
    }

    #[test]
    fn custom_anchor_offsets_aabb_like_local_translation() {
        let anchor = VelloAnchor::CenterOffset(Vec2::new(50.0, 25.0));
        let aabb = anchor.to_aabb_from_dimensions(200.0, 100.0);

        assert_eq!(aabb.min(), Vec3::new(-50.0, -25.0, 0.0).into());
        assert_eq!(aabb.max(), Vec3::new(150.0, 75.0, 0.0).into());
    }
}
