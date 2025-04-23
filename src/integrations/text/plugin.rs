use crate::render::extract::VelloExtractStep;

use super::{VelloFont, VelloTextSection, font_loader::VelloFontLoader, render};
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        render_asset::RenderAssetPlugin,
        view::{VisibilitySystems, check_visibility},
    },
};

pub struct VelloTextIntegrationPlugin;

impl Plugin for VelloTextIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VelloFont>()
            .init_asset_loader::<VelloFontLoader>()
            .add_plugins(RenderAssetPlugin::<VelloFont>::default())
            .add_systems(
                PostUpdate,
                check_visibility::<With<VelloTextSection>>
                    .in_set(VisibilitySystems::CheckVisibility),
            );
        #[cfg(feature = "default_font")]
        {
            let mut fonts = app
                .world_mut()
                .get_resource_mut::<Assets<VelloFont>>()
                .unwrap();

            fonts.insert(
                Handle::default().id(),
                super::font_loader::load_into_font_context(bevy::text::DEFAULT_FONT_DATA.to_vec()),
            );
        }

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(
                ExtractSchedule,
                render::extract_text.in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                Render,
                render::prepare_text_affines.in_set(RenderSet::Prepare),
            );
    }
}
