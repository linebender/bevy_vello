use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSet, render_asset::RenderAssetPlugin},
};

use super::{
    VelloFont,
    font_loader::VelloFontLoader,
    render,
    vello_text::{
        calculate_text_section_content_size_on_change,
        calculate_text_section_content_size_on_screen_scale_change,
        calculate_text_section_content_size_on_world_scale_change,
    },
};
use crate::render::{VelloScreenScale, VelloWorldScale, extract::VelloExtractStep};

pub struct VelloTextIntegrationPlugin;

impl Plugin for VelloTextIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VelloFont>()
            .init_asset_loader::<VelloFontLoader>()
            .add_plugins(RenderAssetPlugin::<VelloFont>::default());

        // PostUpdate is used to ensure that the font handles are available if the consumer
        // of the API is createing a font handle in the same frame.
        app.add_systems(
            PostUpdate,
            (
                calculate_text_section_content_size_on_change,
                calculate_text_section_content_size_on_screen_scale_change
                    .run_if(resource_changed::<VelloScreenScale>),
                calculate_text_section_content_size_on_world_scale_change
                    .run_if(resource_changed::<VelloWorldScale>),
            ),
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
