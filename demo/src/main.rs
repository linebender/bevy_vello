use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_vello::{
    debug::DebugVisualizations, vello_svg::usvg::strict_num::Ulps, AnimationDirection,
    AnimationState, AnimationTransition, ColorPaletteSwap, LottiePlayer, PlaybackSettings, Vector,
    VelloAsset, VelloAssetBundle, VelloPlugin, VelloText, VelloTextBundle,
};

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(VelloPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, (drag_and_drop, print_metadata, ui))
        .run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), bevy_pancam::PanCam::default()));

    commands
        .spawn(VelloAssetBundle {
            origin: bevy_vello::Origin::Center,
            vector: asset_server.load("../assets/squid.json"),
            transform: Transform::from_scale(Vec3::splat(0.5)),
            debug_visualizations: DebugVisualizations::Visible,
            ..default()
        })
        .insert(ColorPaletteSwap::default())
        .insert(
            LottiePlayer::new("Normal")
                .with_state(
                    AnimationState::new("Normal")
                        .with_playback_settings(PlaybackSettings {
                            speed: 0.2,
                            ..default()
                        })
                        .with_transition(AnimationTransition::OnMouseEnter { state: "Reverse" }),
                )
                .with_state(
                    AnimationState::new("Reverse")
                        .with_transition(AnimationTransition::OnMouseLeave { state: "Normal" })
                        .with_playback_settings(PlaybackSettings {
                            speed: 0.2,
                            direction: AnimationDirection::Reverse,
                            ..default()
                        }),
                ),
        );
    commands.spawn(VelloTextBundle {
        font: asset_server.load("../assets/Rubik-Medium.vttf"),
        text: VelloText {
            content: "hello vello".to_string(),
            size: 100.0,
        },
        ..default()
    });

    //commands
    //    .spawn(VelloAssetBundle {
    //        origin: bevy_vello::Origin::Center,
    //        vector: asset_server.load("../assets/example.json"),
    //        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
    //            .with_scale(Vec3::splat(20.0)),
    //        debug_visualizations: DebugVisualizations::Visible,
    //        ..default()
    //    })
    //    .insert(
    //        LottiePlayer::new("stopped")
    //            .with_state(
    //                AnimationState::new("stopped")
    //                    .with_transition(AnimationTransition::OnMouseEnter { state: "play" })
    //                    .with_playback_settings(PlaybackSettings {
    //                        autoplay: false,
    //                        ..default()
    //                    })
    //                    .reset_playhead_on_start(true),
    //            )
    //            .with_state(
    //                AnimationState::new("play")
    //                    .with_transition(AnimationTransition::OnMouseLeave { state: "rev" })
    //                    .with_playback_settings(PlaybackSettings {
    //                        direction: AnimationDirection::Normal,
    //                        looping: AnimationLoopBehavior::None,
    //                        ..default()
    //                    }),
    //            )
    //            .with_state(
    //                AnimationState::new("rev")
    //                    .with_playback_settings(PlaybackSettings {
    //                        direction: AnimationDirection::Reverse,
    //                        ..default()
    //                    })
    //                    .with_transition(AnimationTransition::OnComplete { state: "stopped" }),
    //            ),
    //    );
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
        &mut ColorPaletteSwap,
        &Handle<VelloAsset>,
    )>,
    assets: Res<Assets<VelloAsset>>,
) {
    let Ok((mut player, playback_settings, mut color_swaps, handle)) = player.get_single_mut()
    else {
        return;
    };
    let window = egui::Window::new("Controls")
        .resizable(false)
        .title_bar(true)
        .collapsible(true);

    let asset = assets.get(handle.id()).unwrap();
    let metadata = asset.metadata().unwrap();
    let Vector::Lottie {
        composition,
        first_frame: _,
        rendered_frames: _,
    } = &asset.data
    else {
        return;
    };

    window.show(contexts.ctx_mut(), |ui| {
        ui.heading("Animation Controls");

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

        ui.heading("States");
        let mut transition = None;
        ui.horizontal_wrapped(|ui| {
            for state in player.states() {
                if ui.button(state.id).clicked() {
                    transition.replace(state.id);
                }
            }
        });
        if let Some(transition) = transition {
            player.transition(transition);
        }

        ui.separator();

        ui.heading("Current State");
        ui.label(format!("Autplay: {}", playback_settings.autoplay));
        ui.label(format!("Diration: {:?}", playback_settings.direction));
        ui.label(format!("Intermission: {}", playback_settings.intermission));
        ui.label(format!("Loops: {:?}", playback_settings.looping));
        ui.label(format!("Play mode: {:?}", playback_settings.play_mode));
        ui.label(format!("Segments: {:?}", playback_settings.segments));
        ui.label(format!("Speed: {}", playback_settings.speed));
        ui.collapsing(
            format!("Transitions: {}", player.state().transitions.len()),
            |ui| {
                for transition in player.state().transitions.iter() {
                    ui.label(format!("{transition:?}"));
                }
            },
        );
        ui.separator();

        ui.collapsing("Color Remapping", |ui| {
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
    });
}
