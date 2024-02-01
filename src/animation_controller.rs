use crate::{
    playback_settings::{AnimationLoopBehavior, AnimationPlayMode},
    AnimationDirection, PlaybackSettings, VelloAsset,
};
use bevy::{prelude::*, utils::hashbrown::HashMap};

#[derive(Component, Default, Debug)]
pub struct LottiePlayer {
    initial_state: &'static str,
    current_state: &'static str,
    next_state: Option<&'static str>,
    states: HashMap<&'static str, AnimationState>,
    /// A pending frame to seek to.
    pending_seek_frame: Option<f32>,
    /// A pending duration to change to.
    pending_direction: Option<AnimationDirection>,
    /// A pending intermission to change to.
    pending_intermission: Option<f32>,
    /// A pending loop behavior to change to.
    pending_loop_behavior: Option<AnimationLoopBehavior>,
    /// A pending play mode to change to.
    pending_play_mode: Option<AnimationPlayMode>,
    /// A pending speed to change to.
    pending_speed: Option<f32>,
    /// Whether the player has started.
    started: bool,
    /// Whether the player is playing. State machines will continue unless stopped.
    playing: bool,
    /// Stopped. Doesn't run state machines.
    stopped: bool,
}

impl LottiePlayer {
    /// The current state.
    pub fn state(&self) -> &AnimationState {
        self.states
            .get(self.current_state)
            .unwrap_or_else(|| panic!("state not found: '{}'", self.current_state))
    }

    /// The states in the player
    pub fn states(&self) -> impl Iterator<Item = &AnimationState> {
        self.states.values()
    }

    /// Transition to the next state.
    pub fn transition(&mut self, state: &'static str) {
        self.next_state.replace(state);
    }

    /// Resets or goes back to the default/initial animation.
    pub fn reset(&mut self) {
        self.next_state = Some(self.initial_state);
        self.seek(f32::MIN);
    }

    /// Seeks to a specific frame.
    pub fn seek(&mut self, frame: f32) {
        self.pending_seek_frame = Some(frame);
    }

    /// Sets the player direction. Applies to all animations.
    pub fn set_direction(&mut self, direction: AnimationDirection) {
        self.pending_direction = Some(direction);
    }

    /// Sets the pause between loops. Applies to all animations.
    pub fn set_intermission(&mut self, intermission: f32) {
        self.pending_intermission = Some(intermission);
    }

    /// Sets the loop behavior. Applies to all animations.
    pub fn set_loop_behavior(&mut self, loop_behavior: AnimationLoopBehavior) {
        self.pending_loop_behavior = Some(loop_behavior);
    }

    /// Sets the play mode. Applies to all animations.
    pub fn set_play_mode(&mut self, mode: AnimationPlayMode) {
        self.pending_play_mode = Some(mode);
    }

    /// Sets the animation speed. Applies to all animations.
    pub fn set_speed(&mut self, speed: f32) {
        self.pending_speed = Some(speed);
    }

    /// Toggle the play state.
    pub fn toggle_play(&mut self) {
        if self.stopped || !self.playing {
            self.play();
        } else {
            self.pause();
        }
    }

    /// Play the animation.
    pub fn play(&mut self) {
        self.playing = true;
        self.stopped = false;
    }

    /// Pauses the animation. State machines will continue.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stops the animation. State machines will not run.
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub id: &'static str,
    pub asset: Option<Handle<VelloAsset>>,
    pub playback_settings: Option<PlaybackSettings>,
    pub transitions: Vec<AnimationTransition>,
    /// Whether to reset the playhead when you transition away from this state
    pub reset_playhead_on_transition: bool,
    /// Whether to reset the playhead when the transition it moved to this state
    pub reset_playhead_on_start: bool,
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
            next_state: Some(initial_state),
            pending_seek_frame: None,
            pending_direction: None,
            pending_intermission: None,
            pending_loop_behavior: None,
            pending_play_mode: None,
            pending_speed: None,
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
            reset_playhead_on_transition: false,
            reset_playhead_on_start: false,
        }
    }

    pub fn with_asset(mut self, asset: Handle<VelloAsset>) -> Self {
        self.asset.replace(asset);
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

    pub fn reset_playhead_on_start(mut self, reset: bool) -> Self {
        self.reset_playhead_on_start = reset;
        self
    }
}

pub struct AnimationControllerPlugin;

impl Plugin for AnimationControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PostUpdate,
            (
                systems::apply_player_inputs,
                systems::advance_playheads,
                systems::run_transitions,
                systems::set_state,
            )
                .chain(),
        );
    }
}

pub mod systems {
    use super::{AnimationTransition, LottiePlayer};
    use crate::{AnimationDirection, PlaybackSettings, Vector, VelloAsset};
    use bevy::{prelude::*, utils::Instant};
    use vello_svg::usvg::strict_num::Ulps;

    /// Apply inputs the developer has made, e.g. `player.seek(frame)`
    pub fn apply_player_inputs(
        mut query: Query<(
            &mut LottiePlayer,
            &mut PlaybackSettings,
            &Handle<VelloAsset>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
    ) {
        for (mut player, mut playback_settings, asset_handle) in query.iter_mut() {
            let asset = assets.get_mut(asset_handle.id()).unwrap();
            let VelloAsset {
                data:
                    Vector::Lottie {
                        composition,
                        first_frame: _,
                        rendered_frames,
                    },
                ..
            } = asset
            else {
                continue;
            };

            if let Some(direction) = player.pending_direction.take() {
                for playback_settings in player
                    .states
                    .values_mut()
                    .flat_map(|s| s.playback_settings.as_mut())
                    .chain([playback_settings.as_mut()])
                {
                    playback_settings.direction = direction;
                }
            }
            if let Some(intermission) = player.pending_intermission.take() {
                debug!("changed intermission: {intermission}");
                // This math is particularly hairy. Several things are going on:
                // 1) Preserve the loops completed thus far
                // 2) Do not jump frames
                // 3) Reset the intermission, if inside an intermission
                let length = composition.frames.end - composition.frames.start;
                let loops_completed = {
                    if *rendered_frames > length + playback_settings.intermission {
                        (*rendered_frames / (length + playback_settings.intermission)).trunc()
                    } else if *rendered_frames > length {
                        1.0
                    } else {
                        0.0
                    }
                };
                let in_intermission = *rendered_frames > length
                    && *rendered_frames >= loops_completed * length
                    && *rendered_frames < loops_completed * length + playback_settings.intermission;
                if in_intermission {
                    debug!("in intermission, resetting to {intermission}");
                    *rendered_frames = (loops_completed * (length + intermission)).prev();
                } else {
                    debug!("not in intermission, applying delta to {intermission}");
                    let dt_intermission = intermission - playback_settings.intermission;
                    let dt_frames = dt_intermission * loops_completed;
                    debug!("loops: {loops_completed}, dt_intermission: {dt_intermission}, dt_frames: {dt_frames}");
                    *rendered_frames = (*rendered_frames + dt_frames).max(0.0);
                }
                // Apply
                for playback_settings in player
                    .states
                    .values_mut()
                    .flat_map(|s| s.playback_settings.as_mut())
                    .chain([playback_settings.as_mut()])
                {
                    playback_settings.intermission = intermission;
                }
            }
            if let Some(loop_behavior) = player.pending_loop_behavior.take() {
                // Apply
                for playback_settings in player
                    .states
                    .values_mut()
                    .flat_map(|s| s.playback_settings.as_mut())
                    .chain([playback_settings.as_mut()])
                {
                    playback_settings.looping = loop_behavior;
                }
            }
            if let Some(play_mode) = player.pending_play_mode.take() {
                // Apply
                for playback_settings in player
                    .states
                    .values_mut()
                    .flat_map(|s| s.playback_settings.as_mut())
                    .chain([playback_settings.as_mut()])
                {
                    playback_settings.play_mode = play_mode;
                }
            }
            if let Some(seek_frame) = player.pending_seek_frame.take() {
                let start_frame = playback_settings
                    .segments
                    .start
                    .max(composition.frames.start);
                let end_frame = playback_settings.segments.end.min(composition.frames.end);
                // Bound the seek frame
                let seek_frame = match playback_settings.direction {
                    AnimationDirection::Normal => seek_frame.clamp(start_frame, end_frame),
                    AnimationDirection::Reverse => {
                        end_frame - seek_frame.clamp(start_frame, end_frame)
                    }
                };
                // Preserve the current number of loops when seeking.
                let length = end_frame - start_frame + playback_settings.intermission;
                let loops_completed = (*rendered_frames / length).trunc();
                *rendered_frames = loops_completed * length + seek_frame;
            }
            if let Some(speed) = player.pending_speed.take() {
                // Apply
                for playback_settings in player
                    .states
                    .values_mut()
                    .flat_map(|s| s.playback_settings.as_mut())
                    .chain([playback_settings.as_mut()])
                {
                    playback_settings.speed = speed;
                    error!("{}", playback_settings.speed);
                }
            }
        }
    }

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
                        composition,
                        first_frame, // Set on render
                        rendered_frames,
                    },
                ..
            } = asset
            else {
                continue;
            };

            if first_frame.is_none() {
                first_frame.replace(Instant::now());
                player.started = true;
            }

            // Move frames to control playhead
            let elapsed_frames = dt * playback_settings.speed * composition.frame_rate;
            *rendered_frames += elapsed_frames;
        }
    }

    pub fn set_state(
        mut commands: Commands,
        mut query_sm: Query<(
            Entity,
            &mut LottiePlayer,
            Option<&PlaybackSettings>,
            &mut Handle<VelloAsset>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
    ) {
        for (entity, mut controller, playback_settings, mut cur_handle) in query_sm.iter_mut() {
            let Some(next_state) = controller.next_state.take() else {
                continue;
            };
            info!("animation controller transitioning to={next_state}");

            controller.started = false;
            controller.playing = false;

            let target_state = controller
                .states
                .get(&next_state)
                .unwrap_or_else(|| panic!("state not found: '{}'", next_state));
            let target_handle = target_state.asset.clone().unwrap_or(cur_handle.clone());

            let Some(asset) = assets.get_mut(target_handle.id()) else {
                warn!("Asset not ready for transition... re-queue'ing...");
                controller.next_state.replace(next_state);
                return;
            };

            // Switch to asset
            let changed_assets = cur_handle.id() != target_handle.id();
            *cur_handle = target_handle.clone();

            let playback_settings = playback_settings.cloned().unwrap_or_default();
            let playhead = asset.calculate_playhead(&playback_settings).unwrap();
            // Reset play state
            match &mut asset.data {
                Vector::Svg {
                    original: _,
                    first_frame,
                } => {
                    first_frame.take();
                }
                Vector::Lottie {
                    composition,
                    first_frame,
                    rendered_frames,
                } => {
                    first_frame.take();
                    if controller.state().reset_playhead_on_transition
                        || target_state.reset_playhead_on_start
                        || changed_assets
                    {
                        *rendered_frames = 0.0;
                    } else {
                        // Reset loops
                        // Need to reset to the correct frame - This depends on current direction and next direction.
                        let current_direction = playback_settings.direction;
                        let target_direction = target_state
                            .playback_settings
                            .as_ref()
                            .map(|pb| pb.direction)
                            .unwrap_or(AnimationDirection::Normal);
                        match (current_direction, target_direction) {
                            // Normal -> Reverse
                            (AnimationDirection::Normal, AnimationDirection::Reverse) => {
                                *rendered_frames = (composition.frames.end - playhead)
                                    .min(composition.frames.end.prev());
                            }
                            // Reverse -> Normal
                            (AnimationDirection::Reverse, AnimationDirection::Normal) => {
                                *rendered_frames = playhead;
                            }
                            // Reverse<->Reverse, Normal<->Normal
                            _ => {
                                *rendered_frames %=
                                    composition.frames.end - composition.frames.start;
                                *rendered_frames =
                                    rendered_frames.min(composition.frames.end.prev());
                            }
                        }
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
        mut query_sm: Query<(
            &mut LottiePlayer,
            &PlaybackSettings,
            &GlobalTransform,
            &mut Handle<VelloAsset>,
        )>,
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

        for (mut controller, playback_settings, gtransform, current_asset_handle) in
            query_sm.iter_mut()
        {
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
                            controller.next_state = Some(state);
                            break;
                        }
                    }
                    AnimationTransition::OnComplete { state } => {
                        match &current_asset.data {
                            crate::Vector::Svg {..} => panic!("invalid state: '{}', `OnComplete` is only valid for Lottie files. Use `OnAfter` for SVG.", controller.state().id),
                            crate::Vector::Lottie {
                                composition,
                                rendered_frames, ..
                            } => {
                                if *rendered_frames >= composition.frames.end - composition.frames.start + playback_settings.intermission {
                                    controller.next_state = Some(state);
                                    break;
                                }
                            },
                        };
                    }
                    AnimationTransition::OnMouseEnter { state } => {
                        if is_inside {
                            controller.next_state = Some(state);
                            *hovered = true;
                            break;
                        }
                    }
                    AnimationTransition::OnMouseClick { state } => {
                        if is_inside && buttons.just_pressed(MouseButton::Left) {
                            controller.next_state = Some(state);
                            break;
                        }
                    }
                    AnimationTransition::OnMouseLeave { state } => {
                        if *hovered && !is_inside {
                            controller.next_state = Some(state);
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
                            controller.next_state = Some(state);
                            break;
                        }
                    }
                }
            }
        }
    }
}
