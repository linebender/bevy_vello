use bevy::{asset::AssetMetaCheck, log::LogPlugin, prelude::*};
use bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin,
};
use bevy_vello::{
    debug::DebugVisualizations, vello_svg::usvg::strict_num::Ulps, AnimationDirection,
    AnimationLoopBehavior, AnimationState, AnimationTransition, LottiePlayer, PlaybackSettings,
    Theme, VelloAsset, VelloAssetBundle, VelloAssetData, VelloPlugin, VelloText, VelloTextBundle,
};

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "wgpu=error,naga=warn,bevy_vello=debug".to_owned(),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(VelloPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, (drag_and_drop, print_metadata, ui, text_ui))
        .run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
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
        .insert(Theme::empty())
        .insert(
            LottiePlayer::new("stopped")
                .with_state({
                    AnimationState::new("stopped")
                        .with_transition(AnimationTransition::OnMouseEnter { state: "play" })
                        .with_playback_settings(PlaybackSettings {
                            autoplay: false,
                            ..default()
                        })
                        .reset_playhead_on_start(true)
                })
                .with_state(
                    AnimationState::new("play")
                        .with_transition(AnimationTransition::OnMouseLeave { state: "rev" })
                        .with_playback_settings(PlaybackSettings {
                            looping: AnimationLoopBehavior::None,
                            ..default()
                        }),
                )
                .with_state(
                    AnimationState::new("rev")
                        .with_playback_settings(PlaybackSettings {
                            direction: AnimationDirection::Reverse,
                            looping: AnimationLoopBehavior::None,
                            ..default()
                        })
                        .with_transition(AnimationTransition::OnComplete { state: "stopped" }),
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

/// Drag and drop any SVG or Lottie JSON asset into the window and change the displayed asset
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
        &PlaybackSettings,
        &mut Theme,
        &Handle<VelloAsset>,
    )>,
    assets: Res<Assets<VelloAsset>>,
) {
    let Ok((mut player, playback_settings, mut color_swaps, handle)) = player.get_single_mut()
    else {
        return;
    };

    let asset = assets.get(handle.id()).unwrap();
    let metadata = asset.metadata().unwrap();
    let VelloAssetData::Lottie {
        composition,
        first_frame: _,
        rendered_frames: _,
    } = &asset.data
    else {
        return;
    };

    let window = egui::Window::new("Controls")
        .resizable(false)
        .title_bar(true)
        .collapsible(true);
    window.show(contexts.ctx_mut(), |ui| {
        ui.heading("Lottie Player");

        let mut playhead = asset.calculate_playhead(playback_settings).unwrap();
        ui.horizontal(|ui| {
            ui.label("Playhead");
            if ui
                .add(
                    egui::Slider::new(
                        &mut playhead,
                        playback_settings
                            .segments
                            .start
                            .max(composition.frames.start)
                            ..=playback_settings
                                .segments
                                .end
                                .min(composition.frames.end)
                                .prev(),
                    )
                    .integer(),
                )
                .changed()
            {
                player.pause();
                player.seek(playhead);
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
            if ui.button("Reset").clicked() {
                player.reset();
            }
        });

        ui.separator();

        ui.heading("Player Operations");
        ui.label("Note: Player operations apply to ALL states!");
        ui.horizontal(|ui| {
            ui.label("Set Speed");
            let mut speed = playback_settings.speed;
            if ui.add(egui::Slider::new(&mut speed, 0.05..=2.0)).changed() {
                player.set_speed(speed);
            };
        });
        ui.horizontal(|ui| {
            ui.label("Set Intermission");
            let mut intermission = playback_settings.intermission;
            if ui
                .add(egui::Slider::new(&mut intermission, 0.0..=24.0))
                .changed()
            {
                player.set_intermission(intermission);
            };
        });
        ui.horizontal(|ui| {
            ui.label("Set Loop Behavior");
            let looping = playback_settings.looping;
            if ui
                .radio(looping == AnimationLoopBehavior::None, "None")
                .clicked()
            {
                player.set_loop_behavior(AnimationLoopBehavior::None);
            }
            if ui
                .radio(looping == AnimationLoopBehavior::Loop, "Loop")
                .clicked()
            {
                player.set_loop_behavior(AnimationLoopBehavior::Loop);
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
        ui.label(format!("Autoplay: {}", playback_settings.autoplay));
        ui.label(format!("Direction: {:?}", playback_settings.direction));
        ui.label(format!("Intermission: {}", playback_settings.intermission));
        ui.label(format!("Loop Behavior: {:?}", playback_settings.looping));
        ui.label(format!(
            "Segments: {:?}",
            playback_settings
                .segments
                .start
                .max(composition.frames.start)
                ..playback_settings.segments.end.min(composition.frames.end)
        ));
        ui.label(format!("Speed: {}", playback_settings.speed));
        ui.heading(format!("Transitions: {}", player.state().transitions.len()));
        for transition in player.state().transitions.iter() {
            ui.label(format!("{transition:?}"));
        }

        ui.separator();

        ui.heading("Theme");
        for layer in metadata.get_layers() {
            let color = color_swaps.get_mut(layer).cloned().unwrap_or_default();
            let mut color_edit = [color.r(), color.g(), color.b(), color.a()];
            ui.horizontal(|ui| {
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut color_edit)
                    .changed()
                {
                    let [r, g, b, a] = color_edit;
                    color_swaps.edit(layer, Color::rgba(r, g, b, a));
                };
                ui.label(layer);
            });
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
