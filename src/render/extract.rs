use bevy::{
    prelude::*,
    render::{MainWorld, extract_component::ExtractComponent},
};

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
#[derive(Component, Default)]
pub struct SSRenderTarget(pub Handle<Image>);

impl ExtractComponent for SSRenderTarget {
    type QueryData = &'static SSRenderTarget;

    type QueryFilter = ();

    type Out = Self;

    fn extract_component(
        ss_render_target: bevy::ecs::query::QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self> {
        Some(Self(ss_render_target.0.clone()))
    }
}
