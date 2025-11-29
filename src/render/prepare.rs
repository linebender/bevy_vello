use bevy::prelude::*;
#[cfg(any(feature = "svg", feature = "lottie"))]
use bevy::render::view::ExtractedView;
use vello::kurbo::Affine;

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedAffine(pub Affine);

#[cfg(any(feature = "svg", feature = "lottie"))]
#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedTransform(pub GlobalTransform);

// All extracted bevy_vello render instance types should implement this (RenderAsset, RenderScene,
// RenderText, etc...)
#[cfg(any(feature = "svg", feature = "lottie"))]
pub trait PrepareRenderInstance {
    fn final_transform(&self) -> PreparedTransform;
    fn scene_affine(
        &self,
        view: &ExtractedView,
        world_transform: GlobalTransform,
        viewport_size: UVec2,
        world_scale: f32,
        screen_scale: f32,
    ) -> PreparedAffine;
}
