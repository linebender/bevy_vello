use super::{VelloEntityCountData, VelloFrameProfileData};
use crate::prelude::*;
use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera, extract_component::ExtractComponent,
        sync_world::TemporaryRenderEntity, view::RenderLayers, Extract, MainWorld,
    },
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum VelloExtractStep {
    // Extract renderable types, e.g. SVG, Lottie, Text, Scenes
    ExtractAssets,
    // Synchronize frame data
    SyncData,
}

#[derive(Component, Clone)]
pub struct ExtractedVelloScene {
    pub scene: VelloScene,
    pub transform: GlobalTransform,
    pub ui_node: Option<ComputedNode>,
}

pub fn extract_scenes(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &VelloScene,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_scenes = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (scene, transform, view_visibility, inherited_visibility, ui_node, render_layers) in
        query_scenes.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloScene {
                    transform: *transform,
                    scene: scene.clone(),
                    ui_node: ui_node.cloned(),
                })
                .insert(TemporaryRenderEntity);
            n_scenes += 1;
        }
    }

    frame_data.n_scenes = n_scenes;
}

#[derive(Component, Clone)]
pub struct ExtractedVelloText {
    pub text: VelloTextSection,
    pub text_anchor: VelloTextAnchor,
    pub transform: GlobalTransform,
}

pub fn extract_text(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &VelloTextSection,
                &VelloTextAnchor,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&RenderLayers>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    fonts: Extract<Res<Assets<VelloFont>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_texts = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (text, text_anchor, transform, view_visibility, inherited_visibility, render_layers) in
        query_scenes.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }
        // Skip if font isn't loaded.
        let Some(_font) = fonts.get(text.style.font.id()) else {
            continue;
        };

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloText {
                    text: text.clone(),
                    text_anchor: *text_anchor,
                    transform: *transform,
                })
                .insert(TemporaryRenderEntity);
            n_texts += 1;
        }
    }

    frame_data.n_texts = n_texts;
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
        ss_render_target: bevy::ecs::query::QueryItem<'_, Self::QueryData>,
    ) -> Option<Self> {
        Some(Self(ss_render_target.0.clone()))
    }
}
