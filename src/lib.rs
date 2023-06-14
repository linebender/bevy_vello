use std::ops::RangeInclusive;

use assets::VelloVectorLoader;
use bevy::{
    asset::load_internal_asset, prelude::*, reflect::TypeUuid, sprite::Material2dPlugin,
    utils::HashMap,
};
use debug::DebugVisualizationsPlugin;
use font::VelloFont;
use renderer::VelloRenderPlugin;
mod assets;
mod debug;
mod font;
mod lyon_utils;
mod metadata;
mod renderer;
mod rendertarget;

pub use assets::VelloVector;
pub use debug::DebugVisualizations;

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

#[derive(PartialEq, Component, Default, Copy, Clone, Debug, Reflect, FromReflect)]
#[reflect(Component)]
pub enum Layer {
    Background,
    Shadow,
    #[default]
    Middle,
    Foreground,
}

#[derive(PartialEq, Component, Default, Clone, Debug, Reflect, FromReflect)]
#[reflect(Component)]
/// Add this component to a `VelloVectorBundle` entity to enable runtime color editing.
/// This interface allows swapping colors in a lottie composition by selecting the desired layer
/// and shape and overriding the original color with a new color.
///
/// Only works for layer shapes with fill or stroke elements,
/// which must be fixed (non-animated, for now) and solid-colored (no gradients, for now)
pub struct ColorPaletteSwap {
    colors: HashMap<(String, RangeInclusive<usize>), Color>,
}

impl ColorPaletteSwap {
    pub fn empty() -> Self {
        Self {
            colors: HashMap::default(),
        }
    }

    /// Swap a color for the selected layer and shape combination. `layer_filter` will select any layer which
    /// contains the provided string. Select shapes within the layer with `shape_numbers`. Adding the same swap
    /// (layer,shape) key will override the previous entry's color
    pub fn add(
        mut self,
        layer_filter: &str,
        shape_numbers: RangeInclusive<usize>,
        color: Color,
    ) -> Self {
        self.colors
            .insert((layer_filter.to_string(), shape_numbers), color);
        Self {
            colors: self.colors,
        }
    }

    /// Swap a color for the selected layer and shape combination. `layer_filter` will select any layer which
    /// contains the provided string. Select shapes within the layer with `shape_numbers`. Editing colors with
    /// an existing (layer,shape) key will override the previous entry's color
    pub fn edit(&mut self, layer_filter: &str, shape_numbers: RangeInclusive<usize>, color: Color) {
        self.colors
            .insert((layer_filter.to_string(), shape_numbers), color);
    }
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
