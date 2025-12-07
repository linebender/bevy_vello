use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSystems},
};

use super::{
    PlaybackOptions, VelloLottie, VelloLottieAnchor, asset_loader::VelloLottieLoader, render,
    systems,
};
use crate::{
    integrations::lottie::{UiVelloLottie, VelloLottie2d},
    render::{VelatoRenderer, extract::VelloExtractStep},
};

pub struct LottieIntegrationPlugin;

impl Plugin for LottieIntegrationPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "picking")]
        app.add_plugins(super::picking::LottieWorldPickingPlugin);

        app.init_asset_loader::<VelloLottieLoader>()
            .init_asset::<VelloLottie>()
            .register_type::<VelloLottie2d>()
            .register_type::<UiVelloLottie>()
            .register_type::<VelloLottieAnchor>()
            .register_type::<PlaybackOptions>()
            .add_systems(
                PostUpdate,
                (
                    systems::update_lottie_2d_aabb_on_change
                        .in_set(bevy::camera::visibility::VisibilitySystems::CalculateBounds),
                    systems::update_ui_lottie_content_size_on_change
                        .in_set(bevy::ui::UiSystems::Content),
                ),
            )
            // UI Player
            .add_systems(PostUpdate, systems::advance_playheads::<UiVelloLottie>)
            .add_systems(
                Last,
                (
                    systems::run_time_transitions::<UiVelloLottie>,
                    ApplyDeferred, // Sync point: Events generated via run_time_transitions will be executed.
                    systems::transition_state::<UiVelloLottie>,
                )
                    .chain(),
            )
            // World Player
            .add_systems(PostUpdate, systems::advance_playheads::<VelloLottie2d>)
            .add_systems(
                Last,
                (
                    systems::run_time_transitions::<VelloLottie2d>,
                    systems::transition_state::<VelloLottie2d>,
                )
                    .chain(),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<VelatoRenderer>()
            .add_systems(
                ExtractSchedule,
                (
                    render::extract_world_lottie_assets,
                    render::extract_ui_lottie_assets,
                )
                    .in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                Render,
                (render::prepare_asset_affines).in_set(RenderSystems::Prepare),
            );
    }
}
