use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Color32},
};
use bevy_vello::prelude::*;

#[expect(clippy::type_complexity)]
pub fn controls_ui(
    mut contexts: EguiContexts,
    mut player: Option<
        Single<(
            &mut LottiePlayer,
            &mut Playhead,
            &mut PlaybackOptions,
            &mut Theme,
            &VelloLottieHandle,
        )>,
    >,
    assets: Res<Assets<VelloLottie>>,
) {
    let Some((player, playhead, options, theme, handle)) = player.as_deref_mut() else {
        return;
    };

    let asset = assets.get(handle.id()).unwrap();
    let composition = asset.composition.as_ref();

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
                    options.segments.start.max(composition.frames.start)
                        ..=options.segments.end.min(composition.frames.end),
                ))
                .changed()
            {
                player.pause();
                playhead.seek(frame);
            };
            if player.is_stopped() {
                ui.colored_label(Color32::RED, "stopped");
            } else if !player.is_playing() {
                ui.colored_label(Color32::YELLOW, "paused");
            }
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
            let autoplaying = options.autoplay.to_string();
            if ui
                .checkbox(&mut options.autoplay, autoplaying.to_string())
                .changed()
            {
                player.state_mut().options.as_mut().unwrap().autoplay = options.autoplay;
            };
        });
        ui.vertical(|ui| {
            ui.label("Direction");
            ui.horizontal(|ui| {
                ui.separator();
                if ui
                    .radio_value(&mut options.direction, PlaybackDirection::Normal, "Normal")
                    .changed()
                {
                    player.state_mut().options.as_mut().unwrap().direction = options.direction;
                }
            });
            ui.horizontal(|ui| {
                ui.separator();
                if ui
                    .radio_value(
                        &mut options.direction,
                        PlaybackDirection::Reverse,
                        "Reverse",
                    )
                    .changed()
                {
                    player.state_mut().options.as_mut().unwrap().direction = options.direction;
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label("Intermission");
            let mut intermission = options.intermission.as_secs_f32();
            if ui
                .add(egui::Slider::new(&mut intermission, 0.0..=5.0))
                .changed()
            {
                player.state_mut().options.as_mut().unwrap().intermission =
                    Duration::from_secs_f32(intermission);
                options.intermission = Duration::from_secs_f32(intermission);
            };
        });
        ui.vertical(|ui| {
            ui.label("Play Mode");
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(options.play_mode, PlaybackPlayMode::Normal);
                if ui.radio(selected, "Normal").clicked() {
                    player.state_mut().options.as_mut().unwrap().play_mode =
                        PlaybackPlayMode::Normal;
                    options.play_mode = PlaybackPlayMode::Normal;
                }
            });
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(options.play_mode, PlaybackPlayMode::Bounce);
                if ui.radio(selected, "Bounce").clicked() {
                    player.state_mut().options.as_mut().unwrap().play_mode =
                        PlaybackPlayMode::Bounce;
                    options.play_mode = PlaybackPlayMode::Bounce;
                }
            });
        });
        ui.vertical(|ui| {
            ui.label("Looping");
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(options.looping, PlaybackLoopBehavior::DoNotLoop);
                if ui.radio(selected, "Do not loop").clicked() {
                    player.state_mut().options.as_mut().unwrap().looping =
                        PlaybackLoopBehavior::DoNotLoop;
                    options.looping = PlaybackLoopBehavior::DoNotLoop;
                }
            });
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(options.looping, PlaybackLoopBehavior::Amount(..));
                let mut amt = match options.looping {
                    PlaybackLoopBehavior::Amount(amt) => amt,
                    _ => 0,
                };
                let clicked = ui.radio(selected, "Amount").clicked();
                if ui
                    .add_enabled(selected, egui::Slider::new(&mut amt, 0..=5))
                    .changed()
                    || clicked
                {
                    player.state_mut().options.as_mut().unwrap().looping =
                        PlaybackLoopBehavior::Amount(amt);
                    options.looping = PlaybackLoopBehavior::Amount(amt);
                };
            });
            ui.horizontal(|ui| {
                ui.separator();
                let selected = matches!(options.looping, PlaybackLoopBehavior::Loop);
                if ui.radio(selected, "Loop").clicked() {
                    player.state_mut().options.as_mut().unwrap().looping =
                        PlaybackLoopBehavior::Loop;
                    options.looping = PlaybackLoopBehavior::Loop;
                }
            });
        });

        ui.vertical(|ui| {
            ui.label("Segments");
            ui.horizontal(|ui| {
                ui.separator();
                ui.label("Start");
                let mut start = options.segments.start;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut start,
                            composition.frames.start
                                ..=options.segments.end.min(composition.frames.end),
                        )
                        .integer(),
                    )
                    .changed()
                {
                    player.state_mut().options.as_mut().unwrap().segments.start = start;
                    options.segments.start = start;
                };
            });
            ui.horizontal(|ui| {
                ui.separator();
                ui.label("End");
                let mut end = options.segments.end;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut end,
                            options.segments.start.max(composition.frames.start)
                                ..=composition.frames.end,
                        )
                        .integer(),
                    )
                    .changed()
                {
                    player.state_mut().options.as_mut().unwrap().segments.end = end;
                    options.segments.end = end;
                };
            });
        });
        ui.horizontal(|ui| {
            ui.label("Speed");
            let mut speed = options.speed;
            if ui.add(egui::Slider::new(&mut speed, 0.05..=2.0)).changed() {
                player.state_mut().options.as_mut().unwrap().speed = speed;
                options.speed = speed;
            };
        });

        ui.heading("Theme");
        for layer in composition.get_layers() {
            let color = theme.get_mut(layer).cloned().unwrap_or_default();
            let color = color.to_srgba().to_u8_array();
            let mut color32 =
                Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            ui.horizontal(|ui| {
                if ui.color_edit_button_srgba(&mut color32).changed() {
                    let r = color32.r();
                    let g = color32.g();
                    let b = color32.b();
                    let a = color32.a();
                    player
                        .state_mut()
                        .theme
                        .as_mut()
                        .unwrap()
                        .edit(layer, Color::srgba_u8(r, g, b, a));
                    theme.edit(layer, Color::srgba_u8(r, g, b, a));
                };
                ui.label(layer);
            });
        }

        ui.heading(format!("Transitions: {}", player.state().transitions.len()));
        for transition in player.state().transitions.iter() {
            ui.label(format!("{transition:?}"));
        }
    });
}
