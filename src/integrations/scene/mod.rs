pub(crate) mod render;

mod plugin;
pub(crate) use plugin::SceneIntegrationPlugin;

use bevy::camera::visibility::{self, VisibilityClass};
use bevy::prelude::*;

use crate::VelloRenderSpace;
#[derive(Bundle, Default)]
pub struct VelloSceneBundle {
    /// Scene to render
    pub scene: VelloScene,
    /// A transform to apply to this scene
    pub transform: Transform,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
    /// A bucket into which we group entities for the purposes of visibility.
    pub visibility_class: VisibilityClass,
}

/// A simple newtype component wrapper for [`vello::Scene`] for rendering.
///
/// If you render a [`VelloScene`] based on a [`bevy::ui::Node`] size, you may want to also add
/// [`SkipScaling`] to the entity to prevent scaling the scene beyond the node size.
#[derive(Component, Default, Clone, Deref, DerefMut)]
#[require(VelloRenderSpace, Transform, Visibility, VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<VelloScene>)]
pub struct VelloScene(Box<vello::Scene>);

impl VelloScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<vello::Scene> for VelloScene {
    fn from(scene: vello::Scene) -> Self {
        Self(Box::new(scene))
    }
}
