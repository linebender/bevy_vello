#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] - to add before 1.0
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use bevy::{asset::load_internal_asset, prelude::*, sprite::Material2dPlugin, utils::HashMap};
use font::VelloFont;
use renderer::VelloRenderPlugin;
use std::ops::RangeInclusive;

mod assets;
mod compression;
mod font;
mod metadata;
mod renderer;
mod rendertarget;

// Re-exports
pub use vello_svg;
pub use vellottie;

pub use assets::{
    load_lottie_from_bytes, load_lottie_from_str, load_svg_from_bytes, load_svg_from_str, Vector,
    VelloVector,
};
pub use rendertarget::VelloCanvasMaterial;

pub use assets::VelloVectorLoader;
pub use font::VelloFontLoader;

#[cfg(feature = "debug")]
pub mod debug;

pub struct VelloPlugin;

const SSRT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(2314894693238056781);

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "../shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(VelloRenderPlugin);
        app.init_asset::<VelloVector>()
            .init_asset_loader::<VelloVectorLoader>();
        app.init_asset::<VelloFont>()
            .init_asset_loader::<VelloFontLoader>();
        app.add_plugins((
            Material2dPlugin::<rendertarget::VelloCanvasMaterial>::default(),
            #[cfg(feature = "debug")]
            debug::DebugVisualizationsPlugin,
        ));
        app.add_systems(Startup, rendertarget::setup_ss_rendertarget)
            .add_systems(
                Update,
                (
                    rendertarget::resize_rendertargets,
                    rendertarget::clear_when_empty,
                ),
            );
    }
}

#[derive(PartialEq, Component, Default, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum Layer {
    Background,
    Shadow,
    #[default]
    Ground,
    Foreground,
    UI,
}

#[derive(PartialEq, Component, Default, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum Origin {
    #[default]
    BottomCenter,
    Center,
}

#[derive(PartialEq, Component, Default, Clone, Debug, Reflect)]
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

#[derive(Bundle)]
pub struct VelloVectorBundle {
    pub vector: Handle<VelloVector>,
    /// Configures the draw order within the vello canvas
    pub layer: Layer,
    /// This object's transform local origin. Enable debug visualizations to visualize (red X)
    pub origin: Origin,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    #[cfg(feature = "debug")]
    pub debug_visualizations: debug::DebugVisualizations,
    /// User indication of whether an entity is visible
    /// Algorithmically-computed indication of whether an entity is visible
    //and /// should be extracted for rendering
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for VelloVectorBundle {
    fn default() -> Self {
        Self {
            vector: Default::default(),
            layer: Default::default(),
            origin: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            #[cfg(feature = "debug")]
            debug_visualizations: debug::DebugVisualizations::Visible,
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct VelloText {
    pub content: String,
    pub size: f32,
}

#[derive(Bundle)]
pub struct VelloTextBundle {
    pub font: Handle<VelloFont>,
    pub text: VelloText,
    pub layer: Layer,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    /// Algorithmically-computed indication of whether an entity is visible
    //and /// should be extracted for rendering
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for VelloTextBundle {
    fn default() -> Self {
        Self {
            font: Default::default(),
            text: Default::default(),
            layer: Layer::Foreground,
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
