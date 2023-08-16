use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, Extract},
    window::PrimaryWindow,
};

use crate::{font::VelloFont, ColorPaletteSwap, Layer, VelloText, VelloVector};

#[derive(Component)]
pub struct RenderReadyTag;

pub fn tag_vectors_for_render(
    mut commands: Commands,
    vector_assets: ResMut<Assets<VelloVector>>,
    vectors: Query<(Entity, &Handle<VelloVector>), Without<RenderReadyTag>>,
) {
    for (entity, handle) in vectors.iter() {
        if vector_assets.get(handle).is_some() {
            commands.entity(entity).insert(RenderReadyTag);
        }
    }
}

#[derive(Component, Clone)]
pub struct ExtractedRenderVector {
    pub vector_handle: Handle<VelloVector>,
    pub render_data: VelloVector,
    pub transform: GlobalTransform,
    pub layer: Layer,
    pub color_pallette_swap: Option<ColorPaletteSwap>,
    pub ui_node: Option<Node>,
}

pub fn vector_instances(
    mut commands: Commands,
    query_vectors: Extract<
        Query<(
            &Handle<VelloVector>,
            &Layer,
            &GlobalTransform,
            Option<&ColorPaletteSwap>,
            Option<&Node>,
        )>,
    >,
    assets: Extract<Res<Assets<VelloVector>>>,
) {
    for (vello_vector_handle, layer, transform, color_pallette_swap, ui_node) in
        query_vectors.iter()
    {
        if let Some(asset_data) = assets.get(vello_vector_handle) {
            commands.spawn(ExtractedRenderVector {
                vector_handle: vello_vector_handle.clone(),
                render_data: asset_data.to_owned(),
                transform: *transform,
                layer: *layer,
                color_pallette_swap: color_pallette_swap.cloned(),
                ui_node: ui_node.cloned(),
            });
        }
    }
}

#[derive(Component, Clone)]
pub struct ExtractedRenderText {
    pub font: Handle<VelloFont>,
    pub text: VelloText,
    pub transform: GlobalTransform,
    pub layer: Layer,
}

impl ExtractComponent for ExtractedRenderText {
    type Query = (
        &'static Handle<VelloFont>,
        &'static VelloText,
        &'static GlobalTransform,
        &'static Layer,
    );

    type Filter = ();
    type Out = Self;

    fn extract_component(
        (vello_font_handle, text, transform, layer): bevy::ecs::query::QueryItem<'_, Self::Query>,
    ) -> Option<Self> {
        Some(Self {
            font: vello_font_handle.clone(),
            text: text.clone(),
            transform: *transform,
            layer: *layer,
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
