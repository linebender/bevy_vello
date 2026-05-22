pub(crate) mod render;

mod plugin;
use bevy::camera::primitives::Aabb;
pub(crate) use plugin::SceneIntegrationPlugin;

use bevy::camera::visibility::{self, VisibilityClass};
use bevy::prelude::*;
use imaging::FillRef;
use vello::{kurbo, peniko};

const PATH_TOLERANCE: f64 = 0.1;

/// A renderable scene in the world.
///
/// This is a retained imaging scene that is lowered into the transient frame `vello::Scene`
/// during rendering.
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(Aabb, Transform, Visibility, VisibilityClass)]
#[cfg_attr(feature = "picking", require(Pickable))]
#[component(on_add = visibility::add_visibility_class::<VelloScene2d>)]
pub struct VelloScene2d(Box<imaging::record::Scene>);

impl VelloScene2d {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all recorded commands.
    pub fn reset(&mut self) {
        self.0.clear();
    }

    /// Fill a shape using Vello-like authoring parameters.
    pub fn fill<'a>(
        &mut self,
        fill_rule: peniko::Fill,
        transform: kurbo::Affine,
        brush: impl Into<peniko::BrushRef<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
    ) {
        imaging::PaintSink::fill(
            &mut *self.0,
            FillRef::new(shape.to_path(PATH_TOLERANCE), brush)
                .fill_rule(fill_rule)
                .transform(transform)
                .brush_transform(brush_transform),
        );
    }

    /// Stroke a shape using Vello-like authoring parameters.
    pub fn stroke<'a>(
        &mut self,
        stroke: &'a kurbo::Stroke,
        transform: kurbo::Affine,
        brush: impl Into<peniko::BrushRef<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
    ) {
        imaging::PaintSink::stroke(
            &mut *self.0,
            imaging::StrokeRef::new(shape.to_path(PATH_TOLERANCE), stroke, brush)
                .transform(transform)
                .brush_transform(brush_transform),
        );
    }
}

impl From<imaging::record::Scene> for VelloScene2d {
    fn from(scene: imaging::record::Scene) -> Self {
        Self(Box::new(scene))
    }
}

/// A renderable scene that may be used in Bevy UI.
///
/// This is a retained imaging scene that is lowered into the transient frame `vello::Scene`
/// during rendering.
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(Aabb, UiTransform, Visibility, VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<UiVelloScene>)]
pub struct UiVelloScene(Box<imaging::record::Scene>);

impl UiVelloScene {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all recorded commands.
    pub fn reset(&mut self) {
        self.0.clear();
    }

    /// Fill a shape using Vello-like authoring parameters.
    pub fn fill<'a>(
        &mut self,
        fill_rule: peniko::Fill,
        transform: kurbo::Affine,
        brush: impl Into<peniko::BrushRef<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
    ) {
        imaging::PaintSink::fill(
            &mut *self.0,
            FillRef::new(shape.to_path(PATH_TOLERANCE), brush)
                .fill_rule(fill_rule)
                .transform(transform)
                .brush_transform(brush_transform),
        );
    }

    /// Stroke a shape using Vello-like authoring parameters.
    pub fn stroke<'a>(
        &mut self,
        stroke: &'a kurbo::Stroke,
        transform: kurbo::Affine,
        brush: impl Into<peniko::BrushRef<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
    ) {
        imaging::PaintSink::stroke(
            &mut *self.0,
            imaging::StrokeRef::new(shape.to_path(PATH_TOLERANCE), stroke, brush)
                .transform(transform)
                .brush_transform(brush_transform),
        );
    }
}

impl From<imaging::record::Scene> for UiVelloScene {
    fn from(scene: imaging::record::Scene) -> Self {
        Self(Box::new(scene))
    }
}
