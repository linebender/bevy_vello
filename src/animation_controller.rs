use crate::{
    playback_settings::{AnimationLoopBehavior, AnimationPlayMode},
    AnimationDirection, PlaybackSettings, VelloAsset,
};
use bevy::{
    prelude::*,
    render::{Render, RenderSet},
    utils::hashbrown::HashMap,
};

#[derive(Component, Default, Debug)]
pub struct LottiePlayer {
    initial_state: &'static str,
    current_state: &'static str,
    pending_next_state: Option<&'static str>,
    states: HashMap<&'static str, AnimationState>,
    /// Whether the player has started.
    started: bool,
    /// Whether the player is playing. State machines will continue unless stopped.
    playing: bool,
    /// Stopped. Doesn't run state machines.
    stopped: bool,
}

impl LottiePlayer {
    pub fn state(&self) -> &AnimationState {
        self.states
            .get(self.current_state)
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state))
    }

    pub fn transition(&mut self, state: &'static str) {
        self.pending_next_state.replace(state);
    }

    pub fn reset(&mut self) {
        todo!()
    }

    pub fn seek(&mut self, frame: f32) {
        todo!()
    }

    pub fn set_direction(&mut self, direction: AnimationDirection) {
        todo!()
    }

    pub fn set_loop_behavior(&mut self, loop_behavior: AnimationLoopBehavior) {
        todo!()
    }

    pub fn set_play_mode(&mut self, mode: AnimationPlayMode) {
        todo!()
    }

    pub fn set_speed(&mut self, speed: f32) {
        todo!()
    }

    pub fn toggle_play(&mut self) {
        todo!()
    }

    pub fn play(&mut self) {
        todo!()
    }

    pub fn pause(&mut self) {
        todo!()
    }

    pub fn stop(&mut self) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub id: &'static str,
    pub asset: Handle<VelloAsset>,
    pub playback_settings: Option<PlaybackSettings>,
    pub transitions: Vec<AnimationTransition>,
    pub reset_playhead_on_transition: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum AnimationTransition {
    /// Transitions after a set period of seconds.
    OnAfter {
        state: &'static str,
        secs: f32,
    },
    /// Transition to a different state after all frames complete. Has no effect on SVGs, use `OnAfter` instead.
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
    OnShow {
        state: &'static str,
    },
}

impl LottiePlayer {
    pub fn new(initial_state: &'static str) -> LottiePlayer {
        LottiePlayer {
            initial_state,
            current_state: initial_state,
            pending_next_state: Some(initial_state),
            states: HashMap::new(),
            started: false,
            playing: false,
            stopped: false,
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
            reset_playhead_on_transition: true,
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

    pub fn reset_playhead_on_transition(mut self, reset: bool) -> Self {
        self.reset_playhead_on_transition = reset;
        self
    }
}

pub struct AnimationControllerPlugin;

impl Plugin for AnimationControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                systems::advance_playheads,
                systems::set_state,
                systems::run_transitions,
            )
                .chain(),
        );
    }
}

pub mod systems {
    use super::{AnimationTransition, LottiePlayer};
    use crate::{PlaybackSettings, Vector, VelloAsset};
    use bevy::{prelude::*, utils::Instant};

    /// Advance all the playheads in the scene
    pub fn advance_playheads(
        mut query: Query<(&mut LottiePlayer, &PlaybackSettings, &Handle<VelloAsset>)>,
        mut assets: ResMut<Assets<VelloAsset>>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (mut player, playback_settings, asset_handle) in query.iter_mut() {
            if player.stopped {
                continue;
            }
            // Auto play
            if playback_settings.autoplay && !player.started {
                player.playing = true;
            }
            // Return if paused
            if !player.playing {
                continue;
            }

            // Continue, assuming we are currently playing.
            let asset = assets.get_mut(asset_handle.id()).unwrap();
            let VelloAsset {
                data:
                    Vector::Lottie {
                        original,
                        colored: _,  // Set on render
                        first_frame, // Set on render
                        playhead,
                    },
                ..
            } = asset
            else {
                continue;
            };

            if first_frame.is_none() {
                first_frame.replace(Instant::now());
            }

            // Move playhead
            let elapsed_frames = dt * playback_settings.speed * original.frame_rate;
            *playhead += elapsed_frames;
        }
    }

    pub fn set_state(
        mut commands: Commands,
        mut query_sm: Query<(Entity, &mut LottiePlayer, &mut Handle<VelloAsset>)>,
        mut assets: ResMut<Assets<VelloAsset>>,
    ) {
        for (entity, mut controller, mut cur_handle) in query_sm.iter_mut() {
            let Some(next_state) = controller.pending_next_state.take() else {
                continue;
            };
            info!("animation controller transitioning to={next_state}");
            controller.started = false;
            controller.playing = false;

            let target_state = controller
                .states
                .get(&next_state)
                .unwrap_or_else(|| panic!("state not found: '{}'", next_state));

            if controller.state().asset.id() != target_state.asset.id() {
                *cur_handle = target_state.asset.clone();
            }

            let asset = assets.get_mut(cur_handle.id()).unwrap();
            // Reset play state
            match &mut asset.data {
                Vector::Svg {
                    original: _,
                    first_frame,
                } => {
                    first_frame.take();
                }
                Vector::Lottie {
                    original,
                    colored: _,
                    first_frame,
                    playhead,
                } => {
                    first_frame.take();
                    if controller.state().reset_playhead_on_transition {
                        *playhead = original.frames.start;
                    }
                }
            }

            commands
                .entity(entity)
                .insert(target_state.playback_settings.clone().unwrap_or_default());
            controller.current_state = next_state;
        }
    }

    pub fn run_transitions(
        mut query_sm: Query<(&mut LottiePlayer, &GlobalTransform, &mut Handle<VelloAsset>)>,
        mut assets: ResMut<Assets<VelloAsset>>,

        // For transitions
        windows: Query<&Window>,
        query_view: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
        buttons: Res<Input<MouseButton>>,
        mut hovered: Local<bool>,
    ) {
        let window = windows.single();
        let (camera, view) = query_view.single();

        let pointer_pos = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(view, cursor))
            .map(|ray| ray.origin.truncate());

        for (mut controller, gtransform, current_asset_handle) in query_sm.iter_mut() {
            if controller.stopped {
                continue;
            }

            let current_state_name = controller.current_state.to_owned();
            let current_asset = assets
                .get_mut(current_asset_handle.id())
                .unwrap_or_else(|| panic!("asset not found for state: '{current_state_name}'"));

            let is_inside = {
                match pointer_pos {
                    Some(pointer_pos) => {
                        let local_transform = current_asset
                            .local_transform_center
                            .compute_matrix()
                            .inverse();
                        let transform = gtransform.compute_matrix() * local_transform;
                        let mouse_local = transform
                            .inverse()
                            .transform_point3(pointer_pos.extend(0.0));
                        mouse_local.x <= current_asset.width
                            && mouse_local.x >= 0.0
                            && mouse_local.y >= -current_asset.height
                            && mouse_local.y <= 0.0
                    }
                    None => false,
                }
            };

            for transition in controller.state().transitions.iter() {
                match transition {
                    AnimationTransition::OnAfter { state, secs } => {
                        let started = match current_asset.data {
                            Vector::Svg { first_frame, .. }
                            | Vector::Lottie { first_frame, .. } => first_frame,
                        };
                        if started.is_some_and(|s| s.elapsed().as_secs_f32() > *secs) {
                            controller.pending_next_state = Some(state);
                            break;
                        }
                    }
                    AnimationTransition::OnComplete { state } => {
                        match &current_asset.data {
                            crate::Vector::Svg {..} => warn!("invalid state: '{}', `OnComplete` is only valid for Lottie files. Use `OnAfter` for SVG.", controller.state().id),
                            crate::Vector::Lottie {
                                original: composition,
                                playhead, ..
                            } => {
                                if *playhead >= composition.frames.end {
                                    controller.pending_next_state = Some(state);
                                    break;
                                }
                            },
                        };
                    }
                    AnimationTransition::OnMouseEnter { state } => {
                        if is_inside {
                            controller.pending_next_state = Some(state);
                            break;
                        }
                    }
                    AnimationTransition::OnMouseClick { state } => {
                        if is_inside && buttons.just_pressed(MouseButton::Left) {
                            controller.pending_next_state = Some(state);
                            break;
                        }
                    }
                    AnimationTransition::OnMouseLeave { state } => {
                        if *hovered && !is_inside {
                            controller.pending_next_state = Some(state);
                            *hovered = false;
                            break;
                        } else if is_inside {
                            *hovered = true;
                        }
                    }
                    AnimationTransition::OnShow { state } => {
                        let first_frame = match current_asset.data {
                            Vector::Svg { first_frame, .. }
                            | Vector::Lottie { first_frame, .. } => first_frame,
                        };
                        if first_frame.is_some() {
                            controller.pending_next_state = Some(state);
                            break;
                        }
                    }
                }
            }
        }
    }
}
