use bevy::{
    prelude::*,
    render::{MainWorld, extract_component::ExtractComponent},
    window::PrimaryWindow,
};

use crate::render::VelloPixelScale;

use super::{VelloEntityCountData, VelloFrameProfileData};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum VelloExtractStep {
    // Extract renderable types, e.g. SVG, Lottie, Text, Scenes
    ExtractAssets,
    // Synchronize frame data
    SyncData,
}

/// Synchronize the entity count data back to the main world.
pub fn sync_entity_count(render_data: Res<VelloEntityCountData>, mut world: ResMut<MainWorld>) {
    let mut main_world_data = world.get_resource_mut::<VelloEntityCountData>().unwrap();
    *main_world_data = render_data.clone();
}

/// Synchronize the frame profile data back to the main world.
pub fn sync_frame_profile(render_data: Res<VelloFrameProfileData>, mut world: ResMut<MainWorld>) {
    let mut main_world_data = world.get_resource_mut::<VelloFrameProfileData>().unwrap();
    *main_world_data = render_data.clone();
}

/// A screenspace render target. We use a resizable fullscreen quad.
#[derive(Component, Default, Clone, ExtractComponent)]
pub struct SSRenderTarget(pub Handle<Image>);

pub fn extract_pixel_scale(
    mut pixel_scale: ResMut<VelloPixelScale>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let scale_factor = window.resolution.scale_factor();
    pixel_scale.0 = scale_factor;
}
