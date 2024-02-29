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

#[derive(Component)]
pub struct ExtractedRenderText {
    pub font: VelloFont,
    pub text: VelloText,
    pub transform: GlobalTransform,
    pub render_mode: CoordinateSpace,
}

pub fn text_instances(
    mut commands: Commands,
    query_vectors: Extract<
        Query<(
            &Handle<VelloFont>,
            &VelloText,
            &GlobalTransform,
            &CoordinateSpace,
        )>,
    >,
    assets: Extract<Res<Assets<VelloFont>>>,
) {
    for (vello_font_handle, vello_text, transform, coordinate_space) in
        query_vectors.iter()
    {
        if let Some(asset) = assets.get(vello_font_handle) {
            commands.spawn(ExtractedRenderText {
                font: *asset.to_owned(),
                text: *vello_text,
                transform: *transform,
                render_mode: *coordinate_space,
            });
        }
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
        .map(|window| window.resolution.scale_factor() as f32)
        .unwrap_or(1.0);

    pixel_scale.0 = scale_factor;
}
