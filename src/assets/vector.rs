use crate::{
    lyon_utils::{self, usvg_draw, Convert},
    metadata::Metadata,
};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    math::{Vec3A, Vec4Swizzles},
    prelude::*,
    reflect::TypeUuid,
    render::render_asset::RenderAsset,
    utils::BoxedFuture,
};
use lyon_tessellation::{FillTessellator, StrokeTessellator};
use std::{sync::Arc, time::Instant};
use vello::{SceneBuilder, SceneFragment};
use vello_svg::usvg;

#[derive(Clone)]
pub enum Vector {
    Static(Arc<SceneFragment>),
    Animated(velato::Composition),
}

#[derive(TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-3640-a018b74b5053"]
pub struct VelloVector {
    pub data: Vector,
    pub local_transform: Transform,
    pub width: f32,
    pub height: f32,
    pub tessellation_mesh: Option<Mesh>,
}

impl VelloVector {
    /// Returns the 4 corner points of this vector's bounding box in world space
    pub fn bb_in_world(&self, transform: &GlobalTransform) -> [Vec2; 4] {
        let min = Vec3A::ZERO;
        let x_axis = Vec3A::new(self.width, 0.0, 0.0);

        let max = Vec3A::new(self.width, -self.height, 0.0);
        let y_axis = Vec3A::new(0.0, -self.height, 0.0);

        let world_transform = transform.compute_matrix();
        let local_transform = self.local_transform.compute_matrix().inverse();
        let min = (world_transform * local_transform * min.extend(1.0)).xy();
        let x_axis = (world_transform * local_transform * x_axis.extend(1.0)).xy();
        let max = (world_transform * local_transform * max.extend(1.0)).xy();
        let y_axis = (world_transform * local_transform * y_axis.extend(1.0)).xy();

        [min, x_axis, max, y_axis]
    }

    pub fn metadata(&self) -> Option<Metadata> {
        if let Vector::Animated(composition) = &self.data {
            Some(Metadata {
                composition: composition.clone(),
            })
        } else {
            None
        }
    }
}
