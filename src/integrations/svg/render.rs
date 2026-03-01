use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    render::{Extract, camera::ExtractedCamera, sync_world::TemporaryRenderEntity},
};

use super::{VelloSvgAnchor, asset::VelloSvg};
use crate::{prelude::*, render::VelloEntityCountData};

#[derive(Component, Clone)]
pub struct ExtractedVelloSvg2d {
    pub asset: VelloSvg,
    pub asset_anchor: VelloSvgAnchor,
    pub transform: GlobalTransform,
    pub alpha: f32,
    pub render_layers: RenderLayers,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloSvg {
    pub asset: VelloSvg,
    pub ui_transform: UiGlobalTransform,
    pub alpha: f32,
    pub ui_node: ComputedNode,
    pub render_layers: RenderLayers,
}

pub fn extract_world_svg_assets(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_vectors: Extract<
        Query<
            (
                &VelloSvg2d,
                &VelloSvgAnchor,
                &GlobalTransform,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
            ),
            Without<Node>,
        >,
    >,
    assets: Extract<Res<Assets<VelloSvg>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_svgs = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        asset_handle,
        asset_anchor,
        transform,
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
                .spawn(ExtractedVelloSvg2d {
                    asset: asset.to_owned(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    alpha: asset.alpha,
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_svgs += 1;
        }
    }

    frame_data.n_world_svgs = n_svgs;
}

pub fn extract_ui_svg_assets(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_vectors: Extract<
        Query<(
            &UiVelloSvg,
            &UiGlobalTransform,
            &ComputedNode,
            Option<&RenderLayers>,
            &InheritedVisibility,
        )>,
    >,
    assets: Extract<Res<Assets<VelloSvg>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_svgs = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (asset_handle, ui_transform, ui_node, render_layers, inherited_visibility) in
        query_vectors.iter()
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
                .spawn(ExtractedUiVelloSvg {
                    asset: asset.to_owned(),
                    ui_transform: *ui_transform,
                    ui_node: *ui_node,
                    alpha: asset.alpha,
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_svgs += 1;
        }
    }

    frame_data.n_ui_svgs = n_svgs;
}
