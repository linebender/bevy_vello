use super::z_function::ZFunction;
use crate::text::VelloTextAlignment;
use crate::theme::Theme;
use crate::{
    CoordinateSpace, PlaybackAlphaOverride, Playhead, VelloAsset, VelloFont, VelloScene, VelloText,
};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::Extract;
use bevy::window::PrimaryWindow;

#[derive(Component, Clone)]
pub struct ExtractedRenderAsset {
    pub asset: VelloAsset,
    pub transform: GlobalTransform,
    pub z_index: f32,
    pub theme: Option<Theme>,
    pub render_mode: CoordinateSpace,
    pub playhead: f64,
    pub alpha: f32,
    pub ui_node: Option<Node>,
}

pub fn asset_instances(
    mut commands: Commands,
    query_vectors: Extract<
        Query<(
            &Handle<VelloAsset>,
            &CoordinateSpace,
            &ZFunction,
            &GlobalTransform,
            Option<&Playhead>,
            Option<&Theme>,
            Option<&PlaybackAlphaOverride>,
            Option<&Node>,
            &ViewVisibility,
            &InheritedVisibility,
        )>,
    >,
    assets: Extract<Res<Assets<VelloAsset>>>,
) {
    for (
        vello_vector_handle,
        coord_space,
        z_function,
        transform,
        playhead,
        theme,
        alpha,
        ui_node,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        if let Some(asset) = assets.get(vello_vector_handle) {
            if view_visibility.get() && inherited_visibility.get() {
                let playhead = match asset.data {
                    crate::VectorFile::Svg { .. } => 0.0,
                    crate::VectorFile::Lottie { .. } => playhead.unwrap().frame(),
                };
                commands.spawn(ExtractedRenderAsset {
                    asset: asset.to_owned(),
                    transform: *transform,
                    z_index: z_function.compute(asset, transform),
                    theme: theme.cloned(),
                    render_mode: *coord_space,
                    playhead,
                    alpha: alpha.map(|a| a.0).unwrap_or(1.0),
                    ui_node: ui_node.cloned(),
                });
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct ExtractedRenderScene {
    pub scene: VelloScene,
    pub transform: GlobalTransform,
    pub render_mode: CoordinateSpace,
}

pub fn scene_instances(
    mut commands: Commands,
    query_scenes: Extract<
        Query<(
            &VelloScene,
            &CoordinateSpace,
            &GlobalTransform,
            &ViewVisibility,
            &InheritedVisibility,
        )>,
    >,
) {
    for (scene, coord_space, transform, view_visibility, inherited_visibility) in
        query_scenes.iter()
    {
        if view_visibility.get() && inherited_visibility.get() {
            commands.spawn(ExtractedRenderScene {
                transform: *transform,
                render_mode: *coord_space,
                scene: scene.clone(),
            });
        }
    }
}

#[derive(Component, Clone)]
pub struct ExtractedRenderText {
    pub font: Handle<VelloFont>,
    pub text: VelloText,
    pub alignment: VelloTextAlignment,
    pub transform: GlobalTransform,
    pub render_mode: CoordinateSpace,
}

impl ExtractComponent for ExtractedRenderText {
    type QueryData = (
        &'static Handle<VelloFont>,
        &'static VelloText,
        &'static VelloTextAlignment,
        &'static GlobalTransform,
        &'static CoordinateSpace,
    );

    type QueryFilter = ();

    type Out = Self;

    fn extract_component(
        (vello_font_handle, text, alignment, transform, render_mode): bevy::ecs::query::QueryItem<
            '_,
            Self::QueryData,
        >,
    ) -> Option<Self> {
        Some(Self {
            font: vello_font_handle.clone(),
            text: text.clone(),
            alignment: *alignment,
            transform: *transform,
            render_mode: *render_mode,
        })
    }
}

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
    windows: Extract<Query<&Window, With<PrimaryWindow>>>,
) {
    let scale_factor = windows
        .get_single()
        .map(|window| window.resolution.scale_factor())
        .unwrap_or(1.0);

    pixel_scale.0 = scale_factor;
}
