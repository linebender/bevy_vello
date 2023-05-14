use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid, sprite::Material2dPlugin};
use debug::DebugVisualizationsPlugin;
use font::VelloFont;
use render::VelloRenderPlugin;
use vector::VelloVectorLoader;
mod debug;
mod font;
mod lyon_utils;
mod render;
mod rendertarget;
mod vector;

pub use debug::DebugVisualizations;
pub use vector::VelloVector;

use crate::font::VelloFontLoader;

pub struct BevyVelloPlugin;

const SSRT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2314894693238056781);

impl Plugin for BevyVelloPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "../assets/shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );
        app.add_plugin(VelloRenderPlugin);
        app.add_asset::<VelloVector>()
            .init_asset_loader::<VelloVectorLoader>();
        app.add_asset::<VelloFont>()
            .init_asset_loader::<VelloFontLoader>();
        app.add_plugin(Material2dPlugin::<rendertarget::SSTargetMaterial>::default());
        app.add_plugin(DebugVisualizationsPlugin);
        app.add_systems(Startup, rendertarget::setup_ss_rendertarget)
            .add_systems(Update, rendertarget::resize_rendertargets);
    }
}

#[derive(PartialEq, Component, Default, Copy, Clone, Debug)]
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

#[derive(Component, Default, Clone)]
pub struct VelloText {
    pub content: String,
    pub size: f32,
}

#[derive(Bundle, Default)]
pub struct VelloTextBundle {
    pub font: Handle<VelloFont>,
    pub text: VelloText,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible
    //and /// should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    pub debug_visualizations: DebugVisualizations,
}
