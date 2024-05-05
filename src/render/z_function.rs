use crate::VelloAsset;
use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub enum ZFunction {
    /// Make no adjustment to the asset's transform Z coordinate.
    #[default]
    TransformZ,
    /// Use the asset's X coordinate for Z as well.
    TransformX,
    /// Use the asset's Y coordinate for Z as well.
    TransformY,
    /// Sum the asset's Z coordinate and a constant offset for Z.
    TransformZOffset(f32),
    /// Sum the asset's X coordinate and a constant offset for Z.
    TransformXOffset(f32),
    /// Sum the asset's Y coordinate and a constant offset for Z.
    TransformYOffset(f32),
    /// Use the asset's bounding box top axis value for Z.
    BbTop,
    /// Use the asset's bounding box bottom axis value for Z.
    BbBottom,
    /// Use the asset's bounding box left axis value for Z.
    BbLeft,
    /// Use the asset's bounding box right axis value for Z.
    BbRight,
    /// Use the asset's bounding box top axis value for Z
    BbTopInverse,
    /// Use the asset's bounding box bottom axis value for Z, then flip the sign.
    BbBottomInverse,
    /// Use the asset's bounding box left axis value for Z, then flip the sign.
    BbLeftInverse,
    /// Use the asset's bounding box right axis value for Z, then flip the sign.
    BbRightInverse,
    /// Use a computation to yield Z.
    Computed(fn(&VelloAsset, &GlobalTransform) -> f32),
    /// Use a given value for Z.
    Value(f32),
}

impl ZFunction {
    /// Compute the rendering Z-index using this Z-function.
    pub fn compute(&self, asset: &VelloAsset, transform: &GlobalTransform) -> f32 {
        match self {
            ZFunction::TransformZ => transform.translation().z,
            ZFunction::TransformX => transform.translation().x,
            ZFunction::TransformY => transform.translation().y,
            ZFunction::TransformZOffset(offset) => transform.translation().z + offset,
            ZFunction::TransformXOffset(offset) => transform.translation().x + offset,
            ZFunction::TransformYOffset(offset) => transform.translation().y + offset,
            ZFunction::BbTop => {
                let bb = asset.bb_in_world_space(transform);
                bb.center().y + bb.half_size().y
            }
            ZFunction::BbBottom => {
                let bb = asset.bb_in_world_space(transform);
                bb.center().y - bb.half_size().y
            }
            ZFunction::BbLeft => {
                let bb = asset.bb_in_world_space(transform);
                bb.center().x - bb.half_size().x
            }
            ZFunction::BbRight => {
                let bb = asset.bb_in_world_space(transform);
                bb.center().x + bb.half_size().x
            }
            ZFunction::BbTopInverse => {
                let bb = asset.bb_in_world_space(transform);
                -(bb.center().y + bb.half_size().y)
            }
            ZFunction::BbBottomInverse => {
                let bb = asset.bb_in_world_space(transform);
                -(bb.center().y - bb.half_size().y)
            }
            ZFunction::BbLeftInverse => {
                let bb = asset.bb_in_world_space(transform);
                -(bb.center().x - bb.half_size().x)
            }
            ZFunction::BbRightInverse => {
                let bb = asset.bb_in_world_space(transform);
                -(bb.center().x + bb.half_size().x)
            }
            ZFunction::Computed(compute_fn) => compute_fn(asset, transform),
            ZFunction::Value(v) => *v,
        }
    }
}
