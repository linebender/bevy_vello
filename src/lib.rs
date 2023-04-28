use asset::VelloVectorLoader;
use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid, sprite::Material2dPlugin};
use debug::DebugVisualizationsPlugin;
use render::VelloRenderPlugin;
mod asset;
mod bevy_gizmos;
mod debug;
mod lyon_utils;
mod render;
mod rendertarget;

pub use asset::VelloVector;
pub use debug::DebugVisualizations;

pub struct BevyVelloPlugin;

const SSRT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2314894693238056781);

impl Plugin for BevyVelloPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );
        app.add_plugin(VelloRenderPlugin);
        app.add_plugin(bevy_gizmos::GizmoPlugin);
        app.add_asset::<VelloVector>()
            .init_asset_loader::<VelloVectorLoader>();
        app.add_plugin(Material2dPlugin::<rendertarget::SSTargetMaterial>::default());
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
