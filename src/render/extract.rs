use super::VelloFrameData;
use crate::prelude::*;
use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera, extract_component::ExtractComponent,
        sync_world::TemporaryRenderEntity, view::RenderLayers, Extract, MainWorld,
    },
    window::PrimaryWindow,
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
    pub render_mode: CoordinateSpace,
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
                &CoordinateSpace,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    mut frame_data: ResMut<VelloFrameData>,
) {
    let mut n_scenes = 0;

    // Respect camera ordering
    let mut views: Vec<(&ExtractedCamera, Option<&RenderLayers>)> =
        query_views.into_iter().collect();
    views.sort_by(|(camera_a, _), (camera_b, _)| camera_a.order.cmp(&camera_b.order));

    for (
        scene,
        coord_space,
        transform,
        view_visibility,
        inherited_visibility,
        ui_node,
        render_layers,
    ) in query_scenes.iter()
    {
        if view_visibility.get() && inherited_visibility.get() {
            let asset_render_layers = render_layers.unwrap_or_default();
            for (_, camera_render_layers) in views.iter() {
                if asset_render_layers.intersects(camera_render_layers.unwrap_or_default()) {
                    commands
                        .spawn(ExtractedVelloScene {
                            transform: *transform,
                            render_mode: *coord_space,
                            scene: scene.clone(),
                            ui_node: ui_node.cloned(),
                        })
                        .insert(TemporaryRenderEntity);
                    n_scenes += 1;
                    break;
                }
            }
        }
    }

    frame_data.n_scenes = n_scenes;
}

#[derive(Component, Clone)]
pub struct ExtractedVelloText {
    pub text: VelloTextSection,
    pub text_anchor: VelloTextAnchor,
    pub transform: GlobalTransform,
    pub render_space: CoordinateSpace,
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
                &CoordinateSpace,
                Option<&RenderLayers>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    mut frame_data: ResMut<VelloFrameData>,
) {
    let mut n_texts = 0;

    // Respect camera ordering
    let mut views: Vec<(&ExtractedCamera, Option<&RenderLayers>)> =
        query_views.into_iter().collect();
    views.sort_by(|(camera_a, _), (camera_b, _)| camera_a.order.cmp(&camera_b.order));

    for (
        text,
        text_anchor,
        transform,
        view_visibility,
        inherited_visibility,
        render_space,
        render_layers,
    ) in query_scenes.iter()
    {
        if view_visibility.get() && inherited_visibility.get() {
            let text_render_layers = render_layers.unwrap_or_default();
            for (_, camera_render_layers) in views.iter() {
                if text_render_layers.intersects(camera_render_layers.unwrap_or_default()) {
                    commands
                        .spawn(ExtractedVelloText {
                            text: text.clone(),
                            text_anchor: *text_anchor,
                            transform: *transform,
                            render_space: *render_space,
                        })
                        .insert(TemporaryRenderEntity);
                    n_texts += 1;
                    break;
                }
            }
        }
    }

    frame_data.n_texts = n_texts;
}

/// Synchronize the render world frame data back to the main world.
pub fn sync_frame_data(render_data: Res<VelloFrameData>, mut world: ResMut<MainWorld>) {
    let mut main_world_data = world.get_resource_mut::<VelloFrameData>().unwrap();
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

#[derive(Resource)]
pub struct ExtractedPixelScale(pub f32);

pub fn extract_pixel_scale(
    mut pixel_scale: ResMut<ExtractedPixelScale>,
    window: Extract<Option<Single<&Window, With<PrimaryWindow>>>>,
) {
    let scale_factor = window
        .as_deref()
        .map(|window| window.resolution.scale_factor())
        .unwrap_or(1.0);
    pixel_scale.0 = scale_factor;
}
