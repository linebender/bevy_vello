#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] - TODO add before 1.0
//! An integration to render SVG and Lottie assets in Bevy with Vello.

mod plugin;
pub use plugin::VelloPlugin;

pub mod assets;
pub mod debug;
pub mod metadata;
pub mod playback;
pub mod player;
pub mod renderer;
pub mod rendertarget;
pub mod text;
pub mod theme;

// Re-exports
pub use vello;
pub use vello_svg;
pub use vellottie;

pub mod prelude {
    pub use crate::assets::{VectorFile, VelloAsset};
    pub use crate::debug::DebugVisualizations;
    pub use crate::playback::{
        PlaybackAlphaOverride, PlaybackDirection, PlaybackLoopBehavior,
        PlaybackOptions, PlaybackPlayMode, Playhead,
    };
    pub use crate::player::{LottiePlayer, PlayerState, PlayerTransition};
    pub use crate::plugin::VelloPlugin;
    pub use crate::rendertarget::VelloCanvasMaterial;
    pub use crate::text::{VelloFont, VelloText};
    pub use crate::theme::Theme;
    pub use crate::{CoordinateSpace, VelloAssetBundle, VelloTextBundle};
}

use crate::prelude::*;
use bevy::prelude::*;

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Component,
    Default,
    Copy,
    Clone,
    Debug,
    Reflect,
)]
#[reflect(Component)]
pub enum CoordinateSpace {
    #[default]
    WorldSpace,
    ScreenSpace,
}

#[derive(Bundle, Default)]
pub struct VelloAssetBundle {
    pub vector: Handle<VelloAsset>,
    /// The coordinate space in which this vector should be rendered.
    pub coordinate_space: CoordinateSpace,
    /// A transform to apply to this vector
    pub transform: Transform,
    /// The global transform managed by Bevy
    pub global_transform: GlobalTransform,
    pub debug_visualizations: DebugVisualizations,
    /// User indication of whether an entity is visible. Propagates down the entity hierarchy.
    pub visibility: Visibility,
    /// Whether or not an entity is visible in the hierarchy.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible. Should be extracted for rendering.
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle)]
pub struct VelloTextBundle {
    pub font: Handle<VelloFont>,
    pub text: VelloText,
    /// The coordinate space in which this vector should be rendered.
    pub coordinate_space: CoordinateSpace,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub debug_visualizations: DebugVisualizations,
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
            coordinate_space: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            debug_visualizations: Default::default(),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
