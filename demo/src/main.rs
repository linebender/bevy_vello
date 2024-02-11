use std::time::Duration;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin,
};
use bevy_vello::{
    debug::DebugVisualizations, vello_svg::usvg::strict_num::Ulps,
    LottiePlayer, PlaybackDirection, PlaybackLoopBehavior, PlaybackPlayMode,
    PlaybackSettings, PlayerState, PlayerTransition, Playhead, Theme,
    VectorFile, VelloAsset, VelloAssetBundle, VelloPlugin, VelloText,
    VelloTextBundle,
};

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(VelloPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, (drag_and_drop, print_metadata, ui, text_ui))
        .run();
}

fn setup_vector_graphics(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    commands.spawn((Camera2dBundle::default(), bevy_pancam::PanCam::default()));
    commands.spawn(VelloTextBundle {
        font: asset_server.load("../assets/Rubik-Medium.vttf"),
        text: VelloText {
            content: "hello vello".to_string(),
            size: 100.0,
        },
        ..default()
    });
    commands
        .spawn(VelloAssetBundle {
            vector: asset_server.load("../assets/example.json"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::splat(20.0)),
            debug_visualizations: DebugVisualizations::Visible,
            ..default()
        })
        .insert(
            LottiePlayer::new("stopped")
                .with_state({
                    PlayerState::new("stopped")
                        .with_playback_settings(PlaybackSettings {
                            autoplay: false,
                            ..default()
                        })
                        .with_theme(Theme::new().add("calendar", Color::BLUE))
                        .with_transition(PlayerTransition::OnMouseEnter {
                            state: "play",
                        })
                        .reset_playhead_on_start(true)
                })
                .with_state(
                    PlayerState::new("play")
                        .with_playback_settings(PlaybackSettings {
                            looping: PlaybackLoopBehavior::DoNotLoop,
                            speed: 0.25,
                            ..default()
                        })
                        .with_theme(Theme::new().add("calendar", Color::GREEN))
                        .with_transition(PlayerTransition::OnMouseLeave {
                            state: "rev",
                        }),
                )
                .with_state(
                    PlayerState::new("rev")
                        .with_playback_settings(PlaybackSettings {
                            looping: PlaybackLoopBehavior::DoNotLoop,
                            direction: PlaybackDirection::Reverse,
                            speed: 0.25,
                            ..default()
                        })
                        .with_theme(Theme::new().add("calendar", Color::RED))
                        .with_transition(PlayerTransition::OnMouseEnter {
                            state: "play",
                        })
                        .with_transition(PlayerTransition::OnComplete {
                            state: "stopped",
                        }),
                ),
        );
}

fn print_metadata(
    mut asset_ev: EventReader<AssetEvent<VelloAsset>>,
    assets: Res<Assets<VelloAsset>>,
) {
    for ev in asset_ev.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let asset = assets.get(*id).unwrap();
            if let Some(metadata) = asset.metadata() {
                info!(
                    "Animated asset loaded. Layers:\n{:#?}",
                    metadata.get_layers().collect::<Vec<_>>()
                );
            }
        }
    }
}

/// Drag and drop any SVG or Lottie JSON asset into the window and change the
/// displayed asset
fn drag_and_drop(
    mut query: Query<&mut Handle<VelloAsset>>,
    asset_server: ResMut<AssetServer>,
    mut dnd_evr: EventReader<FileDragAndDrop>,
) {
    let Ok(mut vector) = query.get_single_mut() else {
        return;
    };
    for ev in dnd_evr.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = ev else {
            continue;
        };
        let new_handle = asset_server.load(path_buf.clone());
        *vector = new_handle;
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut player: Query<(
        &mut LottiePlayer,
        &mut Playhead,
        &mut PlaybackSettings,
        &mut Theme,
        &Handle<VelloAsset>,
    )>,
    assets: Res<Assets<VelloAsset>>,
) {
    let Ok((
        mut player,
        mut playhead,
        mut playback_settings,
        mut theme,
        handle,
    )) = player.get_single_mut()
    else {
        return;
    };

    let asset = assets.get(handle.id()).unwrap();
    let metadata = asset.metadata().unwrap();
    let VectorFile::Lottie { composition } = &asset.data else {
        return;
    };

    let window = egui::Window::new("Controls")
        .resizable(false)
        .title_bar(true)
        .collapsible(true);
    window.show(contexts.ctx_mut(), |ui| {
        ui.heading("Lottie Player");

        ui.horizontal(|ui| {
            let mut frame = playhead.frame();
            ui.label("Playhead");
            if ui
                .add(egui::Slider::new(
                    &mut frame,
                    playback_settings
                        .segments
                        .start
                        .max(composition.frames.start)
                        ..=playback_settings
                            .segments
                            .end
                            .min(composition.frames.end)
                            .prev(),
                ))
                .changed()
            {
                player.pause();
                playhead.seek(frame);
            };
        });

        ui.horizontal_wrapped(|ui| {
            if ui.button("Play").clicked() {
                player.play();
            }
            if ui.button("Pause").clicked() {
                player.pause();
            }
            if ui.button("Toggle").clicked() {
                player.toggle_play();
            }
            if ui.button("Stop").clicked() {
                player.stop();
            }
        });

        ui.separator();

        ui.heading("States");
        let mut transition = None;
        ui.horizontal_wrapped(|ui| {
            for state in player.states() {
                let selected = player.state().id == state.id;
                if ui.radio(selected, state.id).clicked() {
                    transition.replace(state.id);
                }
            }
        });
        if let Some(transition) = transition {
            player.transition(transition);
        }

        ui.heading("Current State");
        ui.label(format!("Id: {}", player.state().id));
        ui.horizontal(|ui| {
            ui.label("Autoplay");
            let autoplaying = playback_settings.autoplay.to_string();
            if ui
                .checkbox(
                    &mut playback_settings.autoplay,
                    autoplaying.to_string(),
                )
                .changed()
            {
                player.state_mut().playback_settings.autoplay =
                    playback_settings.autoplay;
            };
        });
        ui.vertical(|ui| {
            ui.label("Direction");
            ui.horizontal(|ui| {
                ui.separator();
                if ui
                    .radio_value(
                        &mut playback_settings.direction,
                        PlaybackDirection::Normal,
                        "Normal",
                    )
                    .changed()
                {
                    player.state_mut().playback_settings.direction =
                        playback_settings.direction;
                }
            });
            ui.horizontal(|ui| {
                ui.separator();
                if ui
                    .radio_value(
                        &mut playback_settings.direction,
                        PlaybackDirection::Reverse,
                        "Reverse",
                    )
                    .changed()
                {
                    player.state_mut().playback_settings.direction =
                        playback_settings.direction;
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label("Intermission");
            let mut intermission = playback_settings.intermission.as_secs_f32();
            if ui
                .add(egui::Slider::new(&mut intermission, 0.0..=5.0))
                .changed()
            {
                player.state_mut().playback_settings.intermission =
                    Duration::from_secs_f32(intermission);
                playback_settings.intermission =
                    Duration::from_secs_f32(intermission);
            };
        });
        ui.vertical(|ui| {
            ui.label("Play Mode");
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(
                    playback_settings.play_mode,
                    PlaybackPlayMode::Normal
                );
                if ui.radio(selected, "Normal").clicked() {
                    player.state_mut().playback_settings.play_mode =
                        PlaybackPlayMode::Normal;
                    playback_settings.play_mode = PlaybackPlayMode::Normal;
                }
            });
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(
                    playback_settings.play_mode,
                    PlaybackPlayMode::Bounce
                );
                if ui.radio(selected, "Bounce").clicked() {
                    player.state_mut().playback_settings.play_mode =
                        PlaybackPlayMode::Bounce;
                    playback_settings.play_mode = PlaybackPlayMode::Bounce;
                }
            });
        });
        ui.vertical(|ui| {
            ui.label("Looping");
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(
                    playback_settings.looping,
                    PlaybackLoopBehavior::DoNotLoop
                );
                if ui.radio(selected, "Do not loop").clicked() {
                    player.state_mut().playback_settings.looping =
                        PlaybackLoopBehavior::DoNotLoop;
                    playback_settings.looping = PlaybackLoopBehavior::DoNotLoop;
                }
            });
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(
                    playback_settings.looping,
                    PlaybackLoopBehavior::Amount(..)
                );
                let mut amt = match playback_settings.looping {
                    PlaybackLoopBehavior::Amount(amt) => amt,
                    _ => 0,
                };
                let clicked = ui.radio(selected, "Amount").clicked();
                if ui
                    .add_enabled(selected, egui::Slider::new(&mut amt, 0..=5))
                    .changed()
                    || clicked
                {
                    player.state_mut().playback_settings.looping =
                        PlaybackLoopBehavior::Amount(amt);
                    playback_settings.looping =
                        PlaybackLoopBehavior::Amount(amt);
                };
            });
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(
                    playback_settings.looping,
                    PlaybackLoopBehavior::Loop
                );
                if ui.radio(selected, "Loop").clicked() {
                    player.state_mut().playback_settings.looping =
                        PlaybackLoopBehavior::Loop;
                    playback_settings.looping = PlaybackLoopBehavior::Loop;
                }
            });
        });

        ui.vertical(|ui| {
            ui.label("Segments");
            ui.horizontal(|ui| {
                ui.separator();
                ui.label("Start");
                let mut start = playback_settings.segments.start;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut start,
                            composition.frames.start
                                ..=playback_settings
                                    .segments
                                    .end
                                    .min(composition.frames.end),
                        )
                        .integer(),
                    )
                    .changed()
                {
                    player.state_mut().playback_settings.segments.start = start;
                    playback_settings.segments.start = start;
                };
            });
            ui.horizontal(|ui| {
                ui.separator();
                ui.label("End");
                let mut end = playback_settings.segments.end;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut end,
                            playback_settings
                                .segments
                                .start
                                .max(composition.frames.start)
                                ..=composition.frames.end,
                        )
                        .integer(),
                    )
                    .changed()
                {
                    player.state_mut().playback_settings.segments.end = end;
                    playback_settings.segments.end = end;
                };
            });
        });
        ui.horizontal(|ui| {
            ui.label("Speed");
            let mut speed = playback_settings.speed;
            if ui.add(egui::Slider::new(&mut speed, 0.05..=2.0)).changed() {
                player.state_mut().playback_settings.speed = speed;
                playback_settings.speed = speed;
            };
        });

        ui.heading("Theme");
        for layer in metadata.get_layers() {
            let color = theme.get_mut(layer).cloned().unwrap_or_default();
            let mut color_edit = [color.r(), color.g(), color.b(), color.a()];
            ui.horizontal(|ui| {
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut color_edit)
                    .changed()
                {
                    let [r, g, b, a] = color_edit;
                    player
                        .state_mut()
                        .theme
                        .edit(layer, Color::rgba(r, g, b, a));
                    theme.edit(layer, Color::rgba(r, g, b, a));
                };
                ui.label(layer);
            });
        }

        ui.heading(format!(
            "Transitions: {}",
            player.state().transitions.len()
        ));
        for transition in player.state().transitions.iter() {
            ui.label(format!("{transition:?}"));
        }
    });
}

fn text_ui(mut contexts: EguiContexts, mut texts: Query<&mut VelloText>) {
    let Ok(mut text) = texts.get_single_mut() else {
        return;
    };

    let window = egui::Window::new("Text")
        .resizable(false)
        .title_bar(true)
        .collapsible(true);
    window.show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Content");
            ui.text_edit_singleline(&mut text.content);
        });
        ui.horizontal(|ui| {
            ui.label("Size");
            ui.add(egui::Slider::new(&mut text.size, 5.0..=200.0));
        });
    });
}
