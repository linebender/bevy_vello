use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSet},
};

use super::{
    PlaybackOptions, VelloLottie, VelloLottieAnchor, asset::VelloLottieHandle,
    asset_loader::VelloLottieLoader, render, systems,
};
use crate::render::{VelatoRenderer, extract::VelloExtractStep};

pub struct LottieIntegrationPlugin;

impl Plugin for LottieIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloLottieLoader>()
            .init_asset::<VelloLottie>()
            .register_type::<VelloLottieHandle>()
            .register_type::<VelloLottieAnchor>()
            .register_type::<PlaybackOptions>()
            .add_systems(PostUpdate, systems::advance_playheads)
            .add_systems(
                Last,
                (systems::run_transitions, systems::transition_state).chain(),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<VelatoRenderer>()
            .add_systems(
                ExtractSchedule,
                render::extract_lottie_assets.in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                Render,
                (render::prepare_asset_affines).in_set(RenderSet::Prepare),
            );
    }
}
