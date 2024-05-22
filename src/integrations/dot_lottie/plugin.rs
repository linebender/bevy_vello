use super::systems;
use bevy::prelude::*;

pub struct DotLottieIntegrationPlugin;

impl Plugin for DotLottieIntegrationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // TODO: Add .lottie loader
        app.add_systems(PostUpdate, systems::advance_dot_lottie_playheads)
            .add_systems(
                Last,
                (systems::run_transitions, systems::transition_state)
                    .chain()
                    .after(crate::integrations::lottie::spawn_playheads),
            );
    }
}
