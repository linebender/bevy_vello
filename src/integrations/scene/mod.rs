pub(crate) mod render;

mod plugin;
use bevy::camera::primitives::Aabb;
pub(crate) use plugin::SceneIntegrationPlugin;

use bevy::camera::visibility::{self, VisibilityClass};
use bevy::prelude::*;

/// A renderable scene in the world.
///
/// A simple newtype component wrapper for [`vello::Scene`].
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(Aabb, Transform, Visibility, VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<VelloScene2d>)]
pub struct VelloScene2d(Box<vello::Scene>);

impl VelloScene2d {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<vello::Scene> for VelloScene2d {
    fn from(scene: vello::Scene) -> Self {
        Self(Box::new(scene))
    }
}

/// A renderable scene that may be used in Bevy UI.
///
/// A simple newtype component wrapper for [`vello::Scene`].
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(Aabb, UiTransform, Visibility, VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<UiVelloScene>)]
pub struct UiVelloScene(Box<vello::Scene>);

impl UiVelloScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<vello::Scene> for UiVelloScene {
    fn from(scene: vello::Scene) -> Self {
        Self(Box::new(scene))
    }
}
