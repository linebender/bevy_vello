use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    render::{Extract, camera::ExtractedCamera, sync_world::TemporaryRenderEntity},
};

use super::{Playhead, Theme, VelloLottieAnchor, asset::VelloLottie};
use crate::integrations::lottie::{UiVelloLottie, VelloLottie2d};
use crate::render::{VelloEntityCountData, VelloView};

#[derive(Component, Clone)]
pub struct ExtractedVelloLottie2d {
    pub asset: VelloLottie,
    pub asset_anchor: VelloLottieAnchor,
    pub transform: GlobalTransform,
    pub alpha: f32,
    pub theme: Option<Theme>,
    pub playhead: f64,
    pub render_layers: RenderLayers,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloLottie {
    pub asset: VelloLottie,
    pub ui_transform: UiGlobalTransform,
    pub alpha: f32,
    pub theme: Option<Theme>,
    pub playhead: f64,
    pub ui_node: ComputedNode,
    pub render_layers: RenderLayers,
}

pub fn extract_world_lottie_assets(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_vectors: Extract<
        Query<
            (
                &VelloLottie2d,
                &VelloLottieAnchor,
                &GlobalTransform,
                &Playhead,
                Option<&Theme>,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
            ),
            Without<Node>,
        >,
    >,
    assets: Extract<Res<Assets<VelloLottie>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_lotties = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        asset_handle,
        asset_anchor,
        transform,
        playhead,
        theme,
        render_layers,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }
        // Skip if asset isn't loaded.
        let Some(asset) = assets.get(asset_handle.id()) else {
            continue;
        };

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.cloned().unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloLottie2d {
                    asset: asset.clone(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    theme: theme.cloned(),
                    playhead: playhead.frame(),
                    alpha: asset.alpha,
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_lotties += 1;
        }
    }

    frame_data.n_world_lotties = n_lotties;
}

pub fn extract_ui_lottie_assets(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_vectors: Extract<
        Query<(
            &UiVelloLottie,
            &UiGlobalTransform,
            &Playhead,
            Option<&Theme>,
            &ComputedNode,
            Option<&RenderLayers>,
            &InheritedVisibility,
        )>,
    >,
    assets: Extract<Res<Assets<VelloLottie>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_lotties = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        asset_handle,
        ui_transform,
        playhead,
        theme,
        ui_node,
        render_layers,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        // Skip if visibility conditions are not met.
        // UI does not check view visibility, only inherited visibility.
        if !inherited_visibility.get() {
            continue;
        }

        // Skip if asset isn't loaded.
        let Some(asset) = assets.get(asset_handle.id()) else {
            continue;
        };

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.cloned().unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedUiVelloLottie {
                    asset: asset.clone(),
                    ui_transform: *ui_transform,
                    theme: theme.cloned(),
                    playhead: playhead.frame(),
                    alpha: asset.alpha,
                    ui_node: *ui_node,
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_lotties += 1;
        }
    }

    frame_data.n_ui_lotties = n_lotties;
}
