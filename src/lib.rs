#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] - to add before 1.0
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use bevy::{asset::load_internal_asset, prelude::*, sprite::Material2dPlugin};
use font::VelloFont;
use renderer::VelloRenderPlugin;

mod assets;
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Component, Default, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum RenderMode {
    #[default]
    WorldSpace = 0,
    ScreenSpace = 1,
}

#[derive(PartialEq, Component, Default, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum Origin {
    #[default]
    BottomCenter,
    Center,
}

#[derive(Bundle)]
pub struct VelloVectorBundle {
    pub vector: Handle<VelloVector>,
    /// The coordinate space in which this vector should be rendered.
    pub render_mode: RenderMode,
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
            render_mode: RenderMode::WorldSpace,
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
    pub render_mode: RenderMode,
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
            render_mode: RenderMode::WorldSpace,
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
