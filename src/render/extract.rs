use super::z_function::ZFunction;
use crate::{
    text::VelloTextAlignment, CoordinateSpace, VelloAsset, VelloAssetAlignment, VelloFont,
    VelloScene, VelloText,
};
use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, Extract},
    window::PrimaryWindow,
};

#[derive(Component, Clone)]
pub struct ExtractedRenderAsset {
    pub asset: VelloAsset,
    pub alignment: VelloAssetAlignment,
    pub transform: GlobalTransform,
    pub z_function: ZFunction,
    #[cfg(feature = "lottie")]
    pub theme: Option<crate::Theme>,
    pub render_mode: CoordinateSpace,
    pub playhead: f64,
    pub alpha: f32,
    pub ui_node: Option<Node>,
}

#[cfg(feature = "svg")]
pub fn extract_svg_instances(
    mut commands: Commands,
    query_vectors: Extract<
        Query<(
            &Handle<VelloAsset>,
            &VelloAssetAlignment,
            &CoordinateSpace,
            &ZFunction,
            &GlobalTransform,
            Option<&Node>,
            &ViewVisibility,
            &InheritedVisibility,
        )>,
    >,
    assets: Extract<Res<Assets<VelloAsset>>>,
) {
    for (
        vello_vector_handle,
        alignment,
        coord_space,
        z_function,
        transform,
        ui_node,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        if let Some(
            asset @ VelloAsset {
                file: _file @ crate::VectorFile::Svg(_),
                alpha,
                ..
            },
        ) = assets.get(vello_vector_handle)
        {
            if view_visibility.get() && inherited_visibility.get() {
                commands.spawn(ExtractedRenderAsset {
                    asset: asset.to_owned(),
                    transform: *transform,
                    alignment: *alignment,
                    z_function: *z_function,
                    #[cfg(feature = "lottie")]
                    theme: None,
                    render_mode: *coord_space,
                    playhead: 0.0,
                    alpha: *alpha,
                    ui_node: ui_node.cloned(),
                });
            }
        }
    }
}

#[cfg(feature = "lottie")]
pub fn extract_lottie_instances(
    mut commands: Commands,
    query_vectors: Extract<
        Query<(
            &Handle<VelloAsset>,
            &VelloAssetAlignment,
            &CoordinateSpace,
            &ZFunction,
            &GlobalTransform,
            &crate::Playhead,
            Option<&crate::Theme>,
            Option<&Node>,
            &ViewVisibility,
            &InheritedVisibility,
        )>,
    >,
    assets: Extract<Res<Assets<VelloAsset>>>,
) {
    for (
        vello_vector_handle,
        alignment,
        coord_space,
        z_function,
        transform,
        playhead,
        theme,
        ui_node,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        if let Some(
            asset @ VelloAsset {
                file: _file @ crate::VectorFile::Lottie(_),
                alpha,
                ..
            },
        ) = assets.get(vello_vector_handle)
        {
            if view_visibility.get() && inherited_visibility.get() {
                let playhead = playhead.frame();
                commands.spawn(ExtractedRenderAsset {
                    asset: asset.to_owned(),
                    transform: *transform,
                    alignment: *alignment,
                    z_function: *z_function,
                    theme: theme.cloned(),
                    render_mode: *coord_space,
                    playhead,
                    alpha: *alpha,
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
    pub ui_node: Option<Node>,
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
            Option<&Node>,
        )>,
    >,
) {
    for (scene, coord_space, transform, view_visibility, inherited_visibility, ui_node) in
        query_scenes.iter()
    {
        error!(
            "view: {}, inherited: {}",
            view_visibility.get(),
            inherited_visibility.get()
        );
        if view_visibility.get() && inherited_visibility.get() {
            commands.spawn(ExtractedRenderScene {
                transform: *transform,
                render_mode: *coord_space,
                scene: scene.clone(),
                ui_node: ui_node.cloned(),
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
