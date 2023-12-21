use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, Extract},
    window::PrimaryWindow,
};

use crate::{font::VelloFont, ColorPaletteSwap, Origin, RenderMode, VelloText, VelloVector};

#[derive(Component, Clone)]
pub struct ExtractedRenderVector {
    pub vector_handle: Handle<VelloVector>,
    pub render_data: VelloVector,
    pub transform: GlobalTransform,
    pub render_mode: RenderMode,
    pub origin: Origin,
    pub color_pallette_swap: Option<ColorPaletteSwap>,
    pub ui_node: Option<Node>,
}

pub fn vector_instances(
    mut commands: Commands,
    query_vectors: Extract<
        Query<(
            &Handle<VelloVector>,
            &RenderMode,
            Option<&Origin>,
            &GlobalTransform,
            Option<&ColorPaletteSwap>,
            Option<&Node>,
            &ViewVisibility,
            &InheritedVisibility,
        )>,
    >,
    assets: Extract<Res<Assets<VelloVector>>>,
) {
    for (
        vello_vector_handle,
        render_mode,
        origin,
        transform,
        color_pallette_swap,
        ui_node,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        if let Some(asset_data) = assets.get(vello_vector_handle) {
            if view_visibility.get() && inherited_visibility.get() {
                commands.spawn(ExtractedRenderVector {
                    vector_handle: vello_vector_handle.clone(),
                    render_data: asset_data.to_owned(),
                    transform: *transform,
                    render_mode: *render_mode,
                    origin: origin.copied().unwrap_or_default(),
                    color_pallette_swap: color_pallette_swap.cloned(),
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
    pub render_mode: RenderMode,
}

impl ExtractComponent for ExtractedRenderText {
    type Query = (
        &'static Handle<VelloFont>,
        &'static VelloText,
        &'static GlobalTransform,
        &'static RenderMode,
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
