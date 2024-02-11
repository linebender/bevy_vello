use super::systems;
use bevy::prelude::*;

pub struct LottiePlayerPlugin;

impl Plugin for LottiePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PreUpdate,
            (systems::spawn_playheads, systems::advance_playheads).chain(),
        )
        .add_systems(
            PostUpdate,
            (systems::run_transitions, systems::transition_state).chain(),
        );
    }
}
