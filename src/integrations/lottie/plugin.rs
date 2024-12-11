use super::{asset_loader::VelloLottieLoader, systems};
use bevy::prelude::*;

pub struct LottieIntegrationPlugin;

impl Plugin for LottieIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloLottieLoader>()
            .add_systems(
                PostUpdate,
                (
                    systems::advance_playheads_without_options,
                    systems::advance_playheads_with_options,
                ),
            )
            .add_observer(systems::spawn_playheads);
    }
}
