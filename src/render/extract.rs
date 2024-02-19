use super::z_function::ZFunction;
use crate::{
    theme::Theme, CoordinateSpace, PlaybackAlphaOverride, Playhead, VelloAsset,
    VelloFont, VelloText,
};
use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, Extract},
    window::PrimaryWindow,
};

#[derive(Component, Clone)]
pub struct ExtractedRenderVector {
    pub asset: VelloAsset,
    pub transform: GlobalTransform,
    pub z_index: f32,
    pub theme: Option<Theme>,
    pub render_mode: CoordinateSpace,
    pub playhead: f32,
    pub alpha: f32,
    pub ui_node: Option<Node>,
}

pub fn vector_instances(
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
                    crate::VectorFile::Lottie { .. } => {
                        playhead.unwrap().frame()
                    }
                };
                commands.spawn(ExtractedRenderVector {
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
pub struct ExtractedRenderText {
    pub font: Handle<VelloFont>,
    pub text: VelloText,
    pub transform: GlobalTransform,
    pub render_mode: CoordinateSpace,
}

impl ExtractComponent for ExtractedRenderText {
    type Query = (
        &'static Handle<VelloFont>,
        &'static VelloText,
        &'static GlobalTransform,
        &'static CoordinateSpace,
    );

    type Filter = ();
    type Out = Self;

    fn extract_component(
        (vello_font_handle, text, transform, render_mode): bevy::ecs::query::QueryItem<
            '_,
            Self::Query,
        >,
    ) -> Option<Self> {
        Some(Self {
            font: vello_font_handle.clone(),
            text: text.clone(),
            transform: *transform,
            render_mode: *render_mode,
        })
    }
}

#[derive(Component, Default)]
pub struct SSRenderTarget(pub Handle<Image>);

impl ExtractComponent for SSRenderTarget {
    type Query = &'static SSRenderTarget;

    type Filter = ();

    type Out = Self;

    fn extract_component(
        ss_render_target: bevy::ecs::query::QueryItem<'_, Self::Query>,
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
        .map(|window| window.resolution.scale_factor() as f32)
        .unwrap_or(1.0);

    pixel_scale.0 = scale_factor;
}
