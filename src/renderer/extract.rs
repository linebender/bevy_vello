use crate::{
    font::VelloFont, player::LottiePlayer, theme::Theme, CoordinateSpace, PlaybackAlphaOverride,
    PlaybackSettings, Playhead, VelloAsset, VelloText,
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
    pub color_swaps: Option<Theme>,
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
            &GlobalTransform,
            Option<&Playhead>,
            Option<&PlaybackSettings>,
            Option<&LottiePlayer>,
            Option<&Theme>,
            Option<&PlaybackAlphaOverride>,
            Option<&Node>,
            &ViewVisibility,
            &InheritedVisibility,
        )>,
    >,
    assets: Extract<Res<Assets<VelloAsset>>>,
    time: Res<Time>,
) {
    for (
        vello_vector_handle,
        render_mode,
        transform,
        playhead,
        playback_settings,
        player,
        color_swaps,
        alpha,
        ui_node,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        if let Some(asset) = assets.get(vello_vector_handle) {
            if view_visibility.get() && inherited_visibility.get() {
                let playhead = playhead
                    .map(|p| p.playhead())
                    .or_else(|| {
                        playback_settings.and_then(|playback_settings| match &asset.data {
                            crate::VelloAssetData::Svg { original: _ } => None,
                            crate::VelloAssetData::Lottie { composition } => {
                                let start_frame = playback_settings
                                    .segments
                                    .start
                                    .max(composition.frames.start);
                                if !playback_settings.autoplay {
                                    Some(start_frame)
                                } else {
                                    let end_frame =
                                        playback_settings.segments.end.min(composition.frames.end);
                                    let length = end_frame - start_frame;
                                    let frame =
                                        (time.elapsed_seconds() * playback_settings.speed) % length;
                                    Some(match playback_settings.direction {
                                        crate::PlaybackDirection::Normal => start_frame + frame,
                                        crate::PlaybackDirection::Reverse => end_frame - frame,
                                    })
                                }
                            }
                        })
                    })
                    .unwrap_or(time.elapsed_seconds());
                commands.spawn(ExtractedRenderVector {
                    asset: asset.to_owned(),
                    transform: *transform,
                    color_swaps: color_swaps.cloned(),
                    render_mode: *render_mode,
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
