use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::camera::ExtractedCamera;
use bevy::render::sync_world::TemporaryRenderEntity;

use crate::integrations::scene::{UiVelloScene, VelloScene2d};
use crate::render::{VelloEntityCountData, VelloView};

#[derive(Component, Clone)]
pub struct ExtractedVelloScene2d {
    pub scene: VelloScene2d,
    pub transform: GlobalTransform,
    pub render_layers: RenderLayers,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloScene {
    pub scene: UiVelloScene,
    pub ui_transform: UiGlobalTransform,
    pub ui_node: ComputedNode,
    pub ui_render_target: ComputedUiRenderTargetInfo,
    pub render_layers: RenderLayers,
}

pub fn extract_world_scenes(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &VelloScene2d,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&RenderLayers>,
            ),
            Without<Node>,
        >,
    >,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_scenes = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (scene, transform, view_visibility, inherited_visibility, render_layers) in
        query_scenes.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.cloned().unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloScene2d {
                    transform: *transform,
                    scene: scene.clone(),
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_scenes += 1;
        }
    }

    frame_data.n_world_scenes = n_scenes;
}

pub fn extract_ui_scenes(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<(
            &UiVelloScene,
            &ComputedNode,
            &ComputedUiRenderTargetInfo,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&RenderLayers>,
        )>,
    >,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_scenes = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (scene, ui_node, ui_render_target, ui_transform, inherited_visibility, render_layers) in
        query_scenes.iter()
    {
        // Skip if visibility conditions are not met.
        // UI does not check view visibility, only inherited visibility.
        if !inherited_visibility.get() {
            continue;
        }
        // Check if any camera renders this asset
        let asset_render_layers = render_layers.cloned().unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedUiVelloScene {
                    scene: scene.clone(),
                    ui_transform: *ui_transform,
                    ui_node: *ui_node,
                    ui_render_target: *ui_render_target,
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_scenes += 1;
        }
    }

    frame_data.n_ui_scenes = n_scenes;
}
