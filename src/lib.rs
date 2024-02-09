#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] - TODO add before 1.0
//! An integration to render SVG and Lottie assets in Bevy with Vello.

mod assets;
mod font;
mod metadata;
mod playback;
mod player;
mod plugin;
mod renderer;
mod rendertarget;
mod theme;

use bevy::prelude::*;
use font::VelloFont;

// Re-exports
pub use vello_svg;
pub use vellottie;

#[cfg(feature = "debug")]
pub mod debug;

pub use assets::VelloAssetLoader;
pub use assets::{
    load_lottie_from_bytes, load_lottie_from_str, load_svg_from_bytes, load_svg_from_str,
    VectorFile, VelloAsset,
};
pub use font::VelloFontLoader;
pub use playback::{
    PlaybackAlphaOverride, PlaybackDirection, PlaybackLoopBehavior, PlaybackSettings, Playhead,
};
pub use player::{LottiePlayer, PlayerState, PlayerTransition};
pub use plugin::VelloPlugin;
pub use rendertarget::VelloCanvasMaterial;
pub use theme::Theme;

#[derive(PartialEq, Eq, PartialOrd, Ord, Component, Default, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum CoordinateSpace {
    #[default]
    WorldSpace = 0,
    ScreenSpace = 1,
}

#[derive(Bundle)]
pub struct VelloAssetBundle {
    pub vector: Handle<VelloAsset>,
    /// The coordinate space in which this vector should be rendered.
    pub coordinate_space: CoordinateSpace,
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

impl Default for VelloAssetBundle {
    fn default() -> Self {
        Self {
            vector: Default::default(),
            coordinate_space: CoordinateSpace::WorldSpace,
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
    pub coordinate_space: CoordinateSpace,
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
            coordinate_space: CoordinateSpace::WorldSpace,
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
