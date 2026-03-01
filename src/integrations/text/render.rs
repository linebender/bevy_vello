use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    render::{Extract, camera::ExtractedCamera, sync_world::TemporaryRenderEntity},
};

use super::{UiVelloText, VelloFont, VelloText2d, VelloTextAnchor};
use crate::render::{VelloEntityCountData, VelloView};

#[derive(Component, Clone)]
pub struct ExtractedVelloText2d {
    pub text: VelloText2d,
    pub text_anchor: VelloTextAnchor,
    pub transform: GlobalTransform,
    pub render_layers: RenderLayers,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloText {
    pub text: UiVelloText,
    pub text_anchor: VelloTextAnchor,
    pub ui_transform: UiGlobalTransform,
    pub ui_node: ComputedNode,
    pub ui_render_target: ComputedUiRenderTargetInfo,
    pub render_layers: RenderLayers,
}

pub fn extract_world_text(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &VelloText2d,
                &VelloTextAnchor,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&RenderLayers>,
            ),
            Without<Node>,
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
        let asset_render_layers = render_layers.cloned().unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloText2d {
                    text: text.clone(),
                    text_anchor: *text_anchor,
                    transform: *transform,
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_texts += 1;
        }
    }

    frame_data.n_world_texts = n_texts;
}

pub fn extract_ui_text(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<(
            &UiVelloText,
            &VelloTextAnchor,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&RenderLayers>,
            &ComputedNode,
            &ComputedUiRenderTargetInfo,
        )>,
    >,
    fonts: Extract<Res<Assets<VelloFont>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_texts = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        text,
        text_anchor,
        ui_transform,
        inherited_visibility,
        render_layers,
        ui_node,
        ui_render_target,
    ) in query_scenes.iter()
    {
        // Skip if visibility conditions are not met.
        // UI does not check view visibility, only inherited visibility.
        if !inherited_visibility.get() {
            continue;
        }

        // Skip if font isn't loaded.
        let Some(_font) = fonts.get(text.style.font.id()) else {
            continue;
        };

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.cloned().unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedUiVelloText {
                    text: text.clone(),
                    text_anchor: *text_anchor,
                    ui_transform: *ui_transform,
                    ui_node: *ui_node,
                    ui_render_target: *ui_render_target,
                    render_layers: asset_render_layers,
                })
                .insert(TemporaryRenderEntity);
            n_texts += 1;
        }
    }

    frame_data.n_ui_texts = n_texts;
}
