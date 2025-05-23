use super::{
    VelloSvg, VelloSvgAnchor, asset::VelloSvgHandle, asset_loader::VelloSvgLoader, render,
};
use crate::render::extract::VelloExtractStep;
use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSet},
};

pub struct SvgIntegrationPlugin;

impl Plugin for SvgIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloSvgLoader>()
            .init_asset::<VelloSvg>()
            .register_type::<VelloSvgHandle>()
            .register_type::<VelloSvgAnchor>();

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
