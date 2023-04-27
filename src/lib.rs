use asset::VelloVectorLoader;
use bevy::{prelude::*, sprite::Material2dPlugin};
use debug::DebugVisualizationsPlugin;
use render::VelloRenderPlugin;
mod asset;
mod debug;
mod lyon_utils;
mod render;
mod rendertarget;

pub use asset::VelloVector;
pub use debug::DebugVisualizations;

pub struct BevyVelloPlugin;

impl Plugin for BevyVelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(VelloRenderPlugin);
        app.add_asset::<VelloVector>()
            .init_asset_loader::<VelloVectorLoader>();
        app.add_plugin(
            Material2dPlugin::<rendertarget::SSTargetMaterial>::default(),
        );
        app.add_plugin(DebugVisualizationsPlugin);
        app.add_startup_system(rendertarget::setup_ss_rendertarget)
            .add_system(rendertarget::resize_rendertargets);
    }
}

#[derive(PartialEq, Component, Default, Copy, Clone)]
pub enum Layer {
    Background,
    Shadow,
    #[default]
    Middle,
    Foreground,
}

#[derive(Bundle, Default)]
pub struct VelloVectorBundle {
    pub svg: Handle<VelloVector>,
    pub layer: Layer,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible
    //and /// should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    pub debug_visualizations: DebugVisualizations,
}
