use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSystems, render_asset::RenderAssetPlugin},
};

use super::{VelloFont, font_loader::VelloFontLoader, render};
use crate::{integrations::text::systems, render::extract::VelloExtractStep};

pub struct VelloTextIntegrationPlugin;

impl Plugin for VelloTextIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VelloFont>()
            .init_asset_loader::<VelloFontLoader>()
            .add_plugins(RenderAssetPlugin::<VelloFont>::default());

        app.add_systems(
            PostUpdate,
            (
                systems::update_text_2d_aabb_on_change
                    .in_set(bevy::camera::visibility::VisibilitySystems::CalculateBounds),
                systems::update_ui_text_content_size_on_change.in_set(bevy::ui::UiSystems::Content),
            ),
        );

        #[cfg(feature = "default_font")]
        {
            let mut fonts = app
                .world_mut()
                .get_resource_mut::<Assets<VelloFont>>()
                .unwrap();

            let _ = fonts.insert(
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
                (render::extract_ui_text, render::extract_world_text)
                    .in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                Render,
                render::prepare_text_affines.in_set(RenderSystems::Prepare),
            );
    }
}
