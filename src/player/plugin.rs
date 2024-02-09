use bevy::prelude::*;

pub struct LottiePlayerPlugin;

impl Plugin for LottiePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PreUpdate,
            (
                systems::spawn_playheads,
                systems::advance_playheads,
                systems::run_transitions,
                systems::transition_state,
            )
                .chain(),
        );
    }
}

pub mod systems {
    use crate::{
        player::LottiePlayer, PlaybackDirection, PlaybackLoopBehavior, PlaybackSettings,
        PlayerTransition, Playhead, VectorFile, VelloAsset,
    };
    use bevy::prelude::*;
    use vello_svg::usvg::strict_num::Ulps;

    /// Spawn playheads for Lotties. Every Lottie gets exactly 1 playhead.
    /// Only
    pub fn spawn_playheads(
        mut commands: Commands,
        query: Query<(Entity, &Handle<VelloAsset>, Option<&PlaybackSettings>), Without<Playhead>>,
        assets: Res<Assets<VelloAsset>>,
    ) {
        for (entity, handle, playback_settings) in query.iter() {
            if let Some(asset) = assets.get(handle) {
                let VectorFile::Lottie { composition } = &asset.data else {
                    commands.entity(entity).insert(Playhead::new(0.0));
                    return;
                };
                let frame = match playback_settings {
                    Some(playback_settings) => match playback_settings.direction {
                        PlaybackDirection::Normal => playback_settings
                            .segments
                            .start
                            .max(composition.frames.start),
                        PlaybackDirection::Reverse => playback_settings
                            .segments
                            .end
                            .min(composition.frames.end)
                            .prev(),
                    },
                    None => composition.frames.start,
                };
                commands.entity(entity).insert(Playhead::new(frame));
            }
        }
    }

    /// Advance all the playheads in the scene
    pub fn advance_playheads(
        mut query: Query<(
            &Handle<VelloAsset>,
            &mut Playhead,
            Option<&mut LottiePlayer>,
            Option<&PlaybackSettings>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
        time: Res<Time>,
    ) {
        for (asset_handle, mut playhead, player, playback_settings) in query.iter_mut() {
            // Get asset
            let Some(VelloAsset {
                data: VectorFile::Lottie { composition },
                ..
            }) = assets.get_mut(asset_handle.id())
            else {
                continue;
            };

            let playback_settings = playback_settings.cloned().unwrap_or_default();
            let start_frame = playback_settings
                .segments
                .start
                .max(composition.frames.start);
            let end_frame = playback_settings
                .segments
                .end
                .min(composition.frames.end)
                .prev();

            if let Some(mut player) = player {
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
            }

            // Advance playhead
            playhead.frame += time.delta_seconds()
                * playback_settings.speed
                * composition.frame_rate
                * (playback_settings.direction as i32 as f32);

            // Keep the playhead bounded between segments
            let looping = match playback_settings.looping {
                PlaybackLoopBehavior::Loop => true,
                PlaybackLoopBehavior::Amount(amt) => amt < playhead.loops_completed,
                PlaybackLoopBehavior::Once => false,
            };
            if playhead.frame > end_frame {
                if looping {
                    // Wrap around to the beginning of the segment
                    playhead.frame = start_frame + (playhead.frame - end_frame);
                    playhead.loops_completed += 1;
                } else {
                    playhead.frame = end_frame;
                }
            } else if playhead.frame < start_frame {
                if looping {
                    // Wrap around to the beginning of the segment
                    playhead.frame = end_frame - (start_frame - playhead.frame);
                    playhead.loops_completed += 1;
                } else {
                    playhead.frame = start_frame;
                }
            }
        }
    }

    pub fn run_transitions(
        mut query_player: Query<(
            &mut LottiePlayer,
            &Playhead,
            &PlaybackSettings,
            &GlobalTransform,
            &mut Handle<VelloAsset>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
        windows: Query<&Window>,
        query_view: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
        buttons: Res<Input<MouseButton>>,
        mut hovered: Local<bool>,
    ) {
        let Ok(window) = windows.get_single() else {
            return;
        };
        let Ok((camera, view)) = query_view.get_single() else {
            return;
        };

        let pointer_pos = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(view, cursor))
            .map(|ray| ray.origin.truncate());

        for (mut player, playhead, playback_settings, gtransform, current_asset_handle) in
            query_player.iter_mut()
        {
            if player.stopped {
                continue;
            }

            let current_state_name = player.current_state.to_owned();
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

            for transition in player.state().transitions.iter() {
                match transition {
                    PlayerTransition::OnAfter { state, secs } => {
                        let started = playhead.first_render;
                        if started.is_some_and(|s| s.elapsed().as_secs_f32() >= *secs) {
                            player.next_state = Some(state);
                            break;
                        }
                    }
                    PlayerTransition::OnComplete { state } => {
                        if let VectorFile::Lottie { composition } = &current_asset.data {
                            match playback_settings.direction {
                                PlaybackDirection::Normal => {
                                    let end_frame = playback_settings
                                        .segments
                                        .end
                                        .min(composition.frames.end)
                                        .prev();
                                    if playhead.frame == end_frame || playhead.loops_completed > 0 {
                                        player.next_state = Some(state);
                                        break;
                                    }
                                }
                                PlaybackDirection::Reverse => {
                                    let start_frame = playback_settings
                                        .segments
                                        .start
                                        .max(composition.frames.start);
                                    if playhead.frame == start_frame || playhead.loops_completed > 0
                                    {
                                        player.next_state = Some(state);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    PlayerTransition::OnMouseEnter { state } => {
                        if is_inside {
                            player.next_state = Some(state);
                            *hovered = true;
                            break;
                        }
                    }
                    PlayerTransition::OnMouseClick { state } => {
                        if is_inside && buttons.just_pressed(MouseButton::Left) {
                            player.next_state = Some(state);
                            break;
                        }
                    }
                    PlayerTransition::OnMouseLeave { state } => {
                        if *hovered && !is_inside {
                            player.next_state = Some(state);
                            *hovered = false;
                            break;
                        } else if is_inside {
                            *hovered = true;
                        }
                    }
                    PlayerTransition::OnShow { state } => {
                        if playhead.first_render.is_some() {
                            player.next_state = Some(state);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn transition_state(
        mut commands: Commands,
        mut query_sm: Query<(
            Entity,
            &mut LottiePlayer,
            &mut Playhead,
            &mut Handle<VelloAsset>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
    ) {
        for (entity, mut player, mut playhead, mut cur_handle) in query_sm.iter_mut() {
            let Some(next_state) = player.next_state.take() else {
                continue;
            };
            info!("animation controller transitioning to={next_state}");

            player.started = false;
            player.playing = false;

            let target_state = player
                .states
                .get(&next_state)
                .unwrap_or_else(|| panic!("state not found: '{}'", next_state));
            let target_handle = target_state.asset.clone().unwrap_or(cur_handle.clone());

            let Some(asset) = assets.get_mut(target_handle.id()) else {
                warn!("Asset not ready for transition... re-queue'ing...");
                player.next_state.replace(next_state);
                return;
            };

            // Switch to asset
            let changed_assets = cur_handle.id() != target_handle.id();
            *cur_handle = target_handle.clone();

            // Reset playhead state
            match &mut asset.data {
                VectorFile::Svg { original: _ } => {
                    // Do nothing
                }
                VectorFile::Lottie { composition } => {
                    if player.state().reset_playhead_on_transition
                        || target_state.reset_playhead_on_start
                        || changed_assets
                    {
                        let playback_settings =
                            target_state.playback_settings.clone().unwrap_or_default();
                        match playback_settings.direction {
                            PlaybackDirection::Normal => {
                                let start_frame = playback_settings
                                    .segments
                                    .start
                                    .max(composition.frames.start);
                                playhead.reset_to(start_frame);
                            }
                            PlaybackDirection::Reverse => {
                                let end_frame = playback_settings
                                    .segments
                                    .end
                                    .min(composition.frames.end)
                                    .prev();
                                playhead.reset_to(end_frame);
                            }
                        }
                    }
                }
            }

            if let Some(theme) = target_state.theme.clone() {
                commands.entity(entity).insert(theme);
            }
            commands
                .entity(entity)
                .insert(target_state.playback_settings.clone().unwrap_or_default());
            player.current_state = next_state;
        }
    }
}