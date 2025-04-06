use super::{
    VelloSvg, VelloSvgAnchor, asset::VelloSvgHandle, asset_loader::VelloSvgLoader, render,
};
use crate::render::extract::VelloExtractStep;
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        view::{VisibilitySystems, check_visibility},
    },
};

pub struct SvgIntegrationPlugin;

impl Plugin for SvgIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloSvgLoader>()
            .init_asset::<VelloSvg>()
            .register_type::<VelloSvgHandle>()
            .register_type::<VelloSvgAnchor>()
            .add_systems(
                PostUpdate,
                check_visibility::<With<VelloSvgHandle>>.in_set(VisibilitySystems::CheckVisibility),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                ExtractSchedule,
                render::extract_svg_assets.in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                Render,
                (render::prepare_asset_affines,).in_set(RenderSet::Prepare),
            );
    }
}
