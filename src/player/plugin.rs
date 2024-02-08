use bevy::prelude::*;

pub struct LottiePlayerPlugin;

impl Plugin for LottiePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PostUpdate,
            (
                systems::advance_playheads,
                systems::run_transitions,
                systems::set_state,
            )
                .chain(),
        );
    }
}

pub mod systems {
    use crate::{
        player::LottiePlayer, PlaybackDirection, PlaybackSettings, VelloAsset, VelloAssetData,
    };
    use bevy::prelude::*;
    use vello_svg::usvg::strict_num::Ulps;

    /// Advance all the playheads in the scene
    pub fn advance_playheads(
        mut query: Query<(
            &Handle<VelloAsset>,
            Option<&mut LottiePlayer>,
            Option<&PlaybackSettings>,
        )>,
        mut assets: ResMut<Assets<VelloAsset>>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (asset_handle, player, playback_settings) in query.iter_mut() {
            // Get asset
            let Some(VelloAsset {
                data: VelloAssetData::Lottie { composition },
                ..
            }) = assets.get_mut(asset_handle.id())
            else {
                continue;
            };

            let playback_settings = playback_settings.cloned().unwrap_or_default();
            let segment_range = playback_settings
                .segments
                .start
                .max(composition.frames.start)
                ..playback_settings.segments.end.min(composition.frames.end);
            let Some(mut player) = player else {
                return;
            };

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

            // Advance playhead
            player.playhead += dt
                * playback_settings.speed
                * composition.frame_rate
                * (playback_settings.direction as i32 as f32);

            // Keep the playhead bounded between segments
            if player.playhead > segment_range.end {
                // Wrap around to the beginning of the segment
                player.playhead = segment_range.start + (player.playhead - segment_range.end);
            } else if player.playhead < segment_range.start {
                // Wrap around to the end of the segment
                player.playhead = segment_range.end - (segment_range.start - player.playhead);
            }
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
                VelloAssetData::Svg {
                    original: _,
                    first_frame,
                } => {
                    first_frame.take();
                }
                VelloAssetData::Lottie {
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
                            .unwrap_or(PlaybackDirection::Normal);
                        match (current_direction, target_direction) {
                            // Normal -> Reverse
                            (PlaybackDirection::Normal, PlaybackDirection::Reverse) => {
                                *rendered_frames = (composition.frames.end - playhead)
                                    .min(composition.frames.end.prev());
                            }
                            // Reverse -> Normal
                            (PlaybackDirection::Reverse, PlaybackDirection::Normal) => {
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

            if let Some(theme) = target_state.theme.clone() {
                commands.entity(entity).insert(theme);
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
                            VelloAssetData::Svg { first_frame, .. }
                            | VelloAssetData::Lottie { first_frame, .. } => first_frame,
                        };
                        if started.is_some_and(|s| s.elapsed().as_secs_f32() >= *secs) {
                            controller.next_state = Some(state);
                            break;
                        }
                    }
                    AnimationTransition::OnComplete { state } => {
                        match &current_asset.data {
                            crate::VelloAssetData::Svg {..} => panic!("invalid state: '{}', `OnComplete` is only valid for Lottie files. Use `OnAfter` for SVG.", controller.state().id),
                            crate::VelloAssetData::Lottie {
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
                            VelloAssetData::Svg { first_frame, .. }
                            | VelloAssetData::Lottie { first_frame, .. } => first_frame,
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
