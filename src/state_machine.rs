use crate::{PlaybackSettings, VelloAsset};
use bevy::{prelude::*, utils::hashbrown::HashMap};

#[derive(Component, Default)]
pub struct AnimationController {
    current_state: &'static str,
    next_state: Option<&'static str>,
    states: HashMap<&'static str, AnimationState>,
}

impl AnimationController {
    pub fn current_state(&self) -> &str {
        self.current_state
    }

    pub fn transition(&mut self, state: &'static str) {
        self.next_state.replace(state);
    }
}

pub struct AnimationState {
    pub id: &'static str,
    pub asset: Handle<VelloAsset>,
    pub playback_settings: Option<PlaybackSettings>,
    pub transitions: Vec<AnimationTransition>,
}

#[allow(clippy::enum_variant_names)]
pub enum AnimationTransition {
    /// Transitions after a set period of seconds.
    OnAfter {
        state: &'static str,
        secs: f32,
    },
    /// Transition to a different state after all frames complete.
    ///
    /// # Panics
    /// Panics if this state transition was attached to an SVG asset, which isn't supported. Use `OnAfter` instead.
    OnComplete {
        state: &'static str,
    },
    OnMouseEnter {
        state: &'static str,
    },
    OnMouseClick {
        state: &'static str,
    },
    OnMouseLeave {
        state: &'static str,
    },
}

impl AnimationController {
    pub fn new(initial_state: &'static str) -> AnimationController {
        AnimationController {
            current_state: initial_state,
            next_state: Some(initial_state),
            states: HashMap::new(),
        }
    }

    pub fn with_state(mut self, state: AnimationState) -> Self {
        self.states.insert(state.id, state);
        self
    }
}

impl AnimationState {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            asset: Default::default(),
            playback_settings: None,
            transitions: vec![],
        }
    }

    pub fn with_asset(mut self, asset: Handle<VelloAsset>) -> Self {
        self.asset = asset;
        self
    }

    pub fn with_playback_settings(mut self, playback_settings: PlaybackSettings) -> Self {
        self.playback_settings.replace(playback_settings);
        self
    }

    pub fn with_transition(mut self, transition: AnimationTransition) -> Self {
        self.transitions.push(transition);
        self
    }
}

pub struct StateMachinePlugin;

impl Plugin for StateMachinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (systems::run_transitions, systems::set_animation_for_state).chain(),
        );
    }
}

pub mod systems {
    use super::{AnimationController, AnimationTransition};
    use crate::{PlaybackSettings, Vector, VelloAsset};
    use bevy::{prelude::*, utils::Instant};

    pub fn set_animation_for_state(
        mut commands: Commands,
        mut query_sm: Query<(Entity, &mut AnimationController, &mut Handle<VelloAsset>)>,
        mut assets: ResMut<Assets<VelloAsset>>,
    ) {
        for (entity, mut state_machine, mut cur_handle) in query_sm.iter_mut() {
            let Some(next_state) = state_machine.next_state.take() else {
                continue;
            };
            let target_state = state_machine
                .states
                .get(&next_state)
                .unwrap_or_else(|| panic!("state not found: '{}'", next_state));

            info!("state machine transitioning to={next_state}");
            let target_asset = assets.get_mut(target_state.asset.id()).unwrap();
            match &mut target_asset.data {
                Vector::Svg {
                    playback_started, ..
                }
                | Vector::Lottie {
                    playback_started, ..
                } => {
                    *playback_started = Instant::now();
                }
            };
            *cur_handle = target_state.asset.clone();
            commands.entity(entity).remove::<PlaybackSettings>();
            if let Some(playback_settings) = &target_state.playback_settings {
                commands.entity(entity).insert(playback_settings.clone());
            }
            state_machine.current_state = next_state;
        }
    }

    pub fn run_transitions(
        mut query_sm: Query<(
            &mut AnimationController,
            Option<&PlaybackSettings>,
            &mut Handle<VelloAsset>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
    ) {
        for (mut state_machine, playback_settings, current_asset_handle) in query_sm.iter_mut() {
            let current_state_name = state_machine.current_state.to_owned();
            let current_asset_id = current_asset_handle.id();

            let current_state = state_machine
                .states
                .get(current_state_name.as_str())
                .unwrap_or_else(|| panic!("state not found: '{current_state_name}'"));
            let current_asset = assets
                .get_mut(current_asset_id)
                .unwrap_or_else(|| panic!("asset not found for state: '{current_state_name}'"));
            for transition in current_state.transitions.iter() {
                match transition {
                    AnimationTransition::OnAfter { state, secs } => {
                        let started = match current_asset.data {
                            Vector::Svg {
                                playback_started, ..
                            }
                            | Vector::Lottie {
                                playback_started, ..
                            } => playback_started,
                        };
                        let elapsed_dt = started.elapsed().as_secs_f32();
                        if elapsed_dt > *secs {
                            state_machine.next_state = Some(state);
                            break;
                        }
                    }
                    AnimationTransition::OnComplete { state } => {
                        match &current_asset.data {
                            crate::Vector::Svg {..} => panic!("invalid state: '{}', `OnComplete` is only valid for Lottie files. Use `OnAfter` for SVG.", current_state.id),
                            crate::Vector::Lottie {
                                original,
                                playback_started, ..
                            } => {
                                let mut elapsed_dt=
                                playback_started.elapsed().as_secs_f32();
                                if let Some(playback_settings) = playback_settings {
                                    elapsed_dt *= playback_settings.speed;
                                }
                                let complete_dt = (original.frames.end - original.frames.start).abs() / original.frame_rate;
                                if elapsed_dt > complete_dt {
                                    state_machine.next_state = Some(state);
                                    break;
                                }
                            },
                        };
                    }
                    AnimationTransition::OnMouseEnter { state } => {
                        todo!("pointer transitions")
                    }
                    AnimationTransition::OnMouseClick { state } => {
                        todo!("pointer transitions")
                    }
                    AnimationTransition::OnMouseLeave { state } => {
                        todo!("pointer transitions")
                    }
                }
            }
        }
    }
}
