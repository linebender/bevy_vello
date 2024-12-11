use crate::{
    integrations::lottie::PlaybackPlayMode, PlaybackDirection, PlaybackLoopBehavior,
    PlaybackOptions, Playhead, VectorFile, VelloAsset, VelloAsset2d,
};
use bevy::{prelude::*, utils::Instant};
use std::time::Duration;
use vello_svg::usvg::strict_num::Ulps;

/// Spawn playheads for Lotties. Every Lottie gets exactly 1 playhead.
/// TODO: This should be a required component method
pub fn spawn_playheads(
    trigger: Trigger<OnAdd, VelloAsset2d>,
    mut commands: Commands,
    query: Query<(&VelloAsset2d, Option<&PlaybackOptions>)>,
    assets: Res<Assets<VelloAsset>>,
) {
    let entity = trigger.entity();
    let Ok((handle, options)) = query.get(entity) else {
        return;
    };
    if let Some(
        _asset @ VelloAsset {
            file: _file @ VectorFile::Lottie(composition),
            ..
        },
    ) = assets.get(handle.id())
    {
        let frame = match options {
            Some(options) => match options.direction {
                PlaybackDirection::Normal => options.segments.start.max(composition.frames.start),
                PlaybackDirection::Reverse => {
                    options.segments.end.min(composition.frames.end).prev()
                }
            },
            None => composition.frames.start,
        };
        commands.entity(entity).insert(Playhead::new(frame));
    }
}

/// Advance all lottie playheads without playback options in the scene
pub fn advance_playheads_without_options(
    #[cfg(feature = "experimental-dotLottie")] mut query: Query<
        (&VelloAsset2d, &mut Playhead),
        (Without<PlaybackOptions>, Without<crate::DotLottiePlayer>),
    >,
    #[cfg(not(feature = "experimental-dotLottie"))] mut query: Query<
        (&VelloAsset2d, &mut Playhead),
        Without<PlaybackOptions>,
    >,
    mut assets: ResMut<Assets<VelloAsset>>,
    time: Res<Time>,
) {
    for (asset_handle, mut playhead) in query.iter_mut() {
        // Get asset
        let Some(VelloAsset {
            file: VectorFile::Lottie(composition),
            ..
        }) = assets.get_mut(asset_handle.id())
        else {
            continue;
        };

        // Keep playhead bounded
        let start_frame = composition.frames.start;
        let end_frame = composition.frames.end.prev();
        playhead.frame = playhead.frame.clamp(start_frame, end_frame);

        // Set first render
        playhead.first_render.get_or_insert(Instant::now());

        // Advance playhead
        let length = end_frame - start_frame;
        playhead.frame += (time.delta_secs_f64() * composition.frame_rate) % length;

        if playhead.frame > end_frame {
            // Wrap around to the beginning of the segment
            playhead.frame = start_frame + (playhead.frame - end_frame);
        }
    }
}

/// Advance all lottie playheads with playback options in the scene
pub fn advance_playheads_with_options(
    #[cfg(feature = "experimental-dotLottie")] mut query: Query<
        (&VelloAsset2d, &mut Playhead, &PlaybackOptions),
        Without<crate::DotLottiePlayer>,
    >,
    #[cfg(not(feature = "experimental-dotLottie"))] mut query: Query<(
        &VelloAsset2d,
        &mut Playhead,
        &PlaybackOptions,
    )>,
    mut assets: ResMut<Assets<VelloAsset>>,
    time: Res<Time>,
) {
    for (asset_handle, mut playhead, options) in query.iter_mut() {
        // Get asset
        let Some(VelloAsset {
            file: VectorFile::Lottie(composition),
            ..
        }) = assets.get_mut(asset_handle.id())
        else {
            continue;
        };

        // Keep playhead bounded
        let start_frame = options.segments.start.max(composition.frames.start);
        let end_frame = options.segments.end.min(composition.frames.end).prev();
        playhead.frame = playhead.frame.clamp(start_frame, end_frame);

        // Set first render
        playhead.first_render.get_or_insert(Instant::now());

        // Auto play
        if !options.autoplay {
            continue;
        }

        // Handle intermissions
        if let Some(ref mut intermission) = playhead.intermission {
            intermission.tick(time.delta());
            if intermission.finished() {
                playhead.intermission.take();
                match options.direction {
                    PlaybackDirection::Normal => {
                        playhead.frame = start_frame;
                    }
                    PlaybackDirection::Reverse => {
                        playhead.frame = end_frame;
                    }
                }
            }
            continue;
        }

        // Advance playhead
        let length = end_frame - start_frame;
        playhead.frame += (time.delta_secs_f64()
            * options.speed
            * composition.frame_rate
            * (options.direction as i32 as f64)
            * playhead.playmode_dir)
            % length;

        // Keep the playhead bounded between segments
        let looping = match options.looping {
            PlaybackLoopBehavior::Loop => true,
            PlaybackLoopBehavior::Amount(amt) => playhead.loops_completed < amt,
            PlaybackLoopBehavior::DoNotLoop => false,
        };
        if playhead.frame > end_frame {
            if looping {
                playhead.loops_completed += 1;
                if let PlaybackPlayMode::Bounce = options.play_mode {
                    playhead.playmode_dir *= -1.0;
                }
                // Trigger intermission, if applicable
                if options.intermission > Duration::ZERO {
                    playhead
                        .intermission
                        .replace(Timer::new(options.intermission, TimerMode::Once));
                    playhead.frame = end_frame;
                } else {
                    // Wrap around to the beginning of the segment
                    playhead.frame = start_frame + (playhead.frame - end_frame);
                }
            } else {
                playhead.frame = end_frame;
            }
            // Obey play mode
            if let PlaybackPlayMode::Bounce = options.play_mode {
                playhead.frame = end_frame;
            }
        } else if playhead.frame < start_frame {
            if looping {
                playhead.loops_completed += 1;
                if let PlaybackPlayMode::Bounce = options.play_mode {
                    playhead.playmode_dir *= -1.0;
                }
                // Trigger intermission, if applicable
                if options.intermission > Duration::ZERO {
                    playhead
                        .intermission
                        .replace(Timer::new(options.intermission, TimerMode::Once));
                    playhead.frame = start_frame;
                } else {
                    // Wrap around to the beginning of the segment
                    playhead.frame = end_frame - (start_frame - playhead.frame);
                }
            } else {
                playhead.frame = start_frame;
            }
            // Obey play mode
            if let PlaybackPlayMode::Bounce = options.play_mode {
                playhead.frame = start_frame;
            }
        }
    }
}
