use crate::{PlaybackSettings, VelloAsset};
use bevy::{prelude::*, utils::HashMap};

#[derive(Component, Default)]
pub struct StateMachine {
    name: &'static str,
    current_state: &'static str,
    states: HashMap<&'static str, State>,
}

pub struct State {
    pub name: &'static str,
    pub asset: Handle<VelloAsset>,
    pub playback_settings: Option<PlaybackSettings>,
    pub transitions: Vec<StateTransition>,
}

#[allow(clippy::enum_variant_names)]
pub enum StateTransition {
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

impl StateMachine {
    pub fn new(name: &'static str, initial_state: &'static str) -> StateMachine {
        StateMachine {
            name,
            current_state: initial_state,
            states: HashMap::new(),
        }
    }

    pub fn with_state(mut self, state: State) -> Self {
        self.states.insert(state.name, state);
        self
    }
}

impl State {
    pub fn new(name: &'static str, asset: Handle<VelloAsset>) -> Self {
        Self {
            name,
            asset,
            playback_settings: None,
            transitions: vec![],
        }
    }

    pub fn with_playback_settings(mut self, playback_settings: PlaybackSettings) -> Self {
        self.playback_settings.replace(playback_settings);
        self
    }

    pub fn with_transition(mut self, transition: StateTransition) -> Self {
        self.transitions.push(transition);
        self
    }
}

pub struct StateMachinePlugin;

impl Plugin for StateMachinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (systems::run_transitions, systems::conform_states).chain(),
        );
    }
}

pub mod systems {
    use super::{StateMachine, StateTransition};
    use crate::{PlaybackSettings, Vector, VelloAsset};
    use bevy::{prelude::*, utils::Instant};

    pub fn conform_states(
        mut commands: Commands,
        mut query_sm: Query<
            (Entity, &StateMachine, Option<&PlaybackSettings>),
            With<Handle<VelloAsset>>,
        >,
    ) {
        for (entity, state_machine, cur_playback_settings) in query_sm.iter_mut() {
            let current_state = state_machine
                .states
                .get(state_machine.current_state)
                .unwrap_or_else(|| {
                    panic!(
                        "({}) state not found: '{}'",
                        state_machine.name, state_machine.current_state
                    )
                });
            match &current_state.playback_settings {
                Some(playback_settings) => {
                    commands.entity(entity).insert(playback_settings.clone());
                }
                None => {
                    commands.entity(entity).remove::<PlaybackSettings>();
                }
            }
        }
    }

    pub fn run_transitions(
        mut query_sm: Query<(
            &mut StateMachine,
            Option<&PlaybackSettings>,
            &mut Handle<VelloAsset>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
    ) {
        for (mut state_machine, playback_settings, mut current_asset_handle) in query_sm.iter_mut()
        {
            let id = current_asset_handle.id();
            let mut target_state = None;

            let current_state = state_machine
                .states
                .get(state_machine.current_state)
                .unwrap_or_else(|| {
                    panic!(
                        "({}) state not found: '{}'",
                        state_machine.name, state_machine.current_state
                    )
                });
            for transition in current_state.transitions.iter() {
                let current_asset = assets.get_mut(id).unwrap_or_else(|| {
                    panic!(
                        "({}) asset not found for state: '{}': '{}'",
                        state_machine.name, state_machine.current_state, id
                    )
                });
                match transition {
                    StateTransition::OnAfter { state, secs } => {
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
                            target_state.replace(state);
                            break;
                        }
                    }
                    StateTransition::OnComplete { state } => {
                        match &current_asset.data {
                            crate::Vector::Svg {..} => panic!("({}) `OnComplete` transition was attached to an SVG asset, which isn't supported. Use `OnAfter` instead.", state_machine.name),
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
                                    target_state.replace(state);
                                    break;
                                }
                            },
                        };
                    }
                    StateTransition::OnMouseEnter { state } => {
                        todo!("pointer transitions")
                    }
                    StateTransition::OnMouseClick { state } => {
                        todo!("pointer transitions")
                    }
                    StateTransition::OnMouseLeave { state } => {
                        todo!("pointer transitions")
                    }
                }
            }

            if let Some(state) = target_state {
                info!(
                    "state machine transitioning. from={}, to={}",
                    state_machine.current_state, state
                );
                let target_state = state_machine.states.get(state).unwrap_or_else(|| {
                    panic!("({}) state not found: '{state}'", state_machine.name)
                });
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
                *current_asset_handle = target_state.asset.clone();
                state_machine.current_state = state;
            }
        }
    }
}
