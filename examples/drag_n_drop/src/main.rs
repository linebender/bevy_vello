use std::{ffi::OsStr, task::Poll};

use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    color::palettes::css::RED,
    prelude::*,
};
use bevy_async_task::TaskRunner;
use bevy_vello::{VelloPlugin, prelude::*};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_plugins(bevy_pancam::PanCamPlugin)
    .add_systems(Startup, setup_camera)
    .add_systems(Startup, setup_initial_image)
    .add_systems(Startup, setup_button)
    .add_systems(Update, (drag_and_drop, button_system))
    .add_observer(cleanup_scene);

    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn setup_initial_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        VelloSvg2d(asset_server.load("embedded://drag_n_drop/assets/Ghostscript_Tiger.svg")),
        Transform::from_scale(Vec3::splat(0.5)),
    ));
}

fn setup_button(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Open"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
        });
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut task_runner: TaskRunner<Option<(String, Vec<u8>)>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = HOVERED_BUTTON.into();
                border_color.set_all(RED);

                let fut = async move {
                    let handle = rfd::AsyncFileDialog::new().pick_file().await?;
                    let file_name = handle.file_name();
                    let file_contents = handle.read().await;
                    Some((file_name, file_contents))
                };
                task_runner.start(fut);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.set_all(Color::WHITE);
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.set_all(Color::BLACK);
            }
        }
    }

    if let Poll::Ready(Some((file_name, file))) = task_runner.poll() {
        load_file(&file_name, &file, &mut commands, &asset_server);
    }
}

fn load_file(file_name: &str, file: &[u8], commands: &mut Commands, asset_server: &AssetServer) {
    if file_name.ends_with(".svg") {
        match bevy_vello::integrations::svg::load_svg_from_bytes(file) {
            Ok(svg) => {
                let handle = asset_server.add(svg);
                commands.trigger(CleanupEvent);
                commands.spawn(VelloSvg2d(handle));
            }
            Err(e) => {
                error!("Failed to load SVG: {e:?}");
            }
        }
    } else if file_name.ends_with(".json") {
        match bevy_vello::integrations::lottie::load_lottie_from_bytes(file) {
            Ok(lottie) => {
                let handle = asset_server.add(lottie);
                commands.trigger(CleanupEvent);
                commands.spawn(VelloLottie2d(handle));
            }
            Err(e) => {
                error!("Failed to load Lottie: {e:?}");
            }
        }
    }
}

/// Drag and drop any SVG or Lottie JSON asset into the window to change the displayed asset
fn drag_and_drop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut dnd_evr: MessageReader<FileDragAndDrop>,
) {
    for ev in dnd_evr.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = ev else {
            continue;
        };

        let Some(ext) = path_buf.extension() else {
            continue;
        };

        match ext {
            ext if ext == OsStr::new("svg") => {
                commands.trigger(CleanupEvent);
                commands.spawn(VelloSvg2d(asset_server.load(path_buf.clone())));
            }
            ext if ext == OsStr::new("json") => {
                commands.trigger(CleanupEvent);
                commands.spawn(VelloLottie2d(asset_server.load(path_buf.clone())));
            }
            _ => continue,
        }
    }
}

#[derive(Event)]
struct CleanupEvent;

fn cleanup_scene(
    _trigger: On<CleanupEvent>,
    mut commands: Commands,
    query_lottie: Option<Single<Entity, With<VelloLottie2d>>>,
    query_svg: Option<Single<Entity, With<VelloSvg2d>>>,
) {
    if let Some(svg) = query_svg {
        commands.entity(*svg).despawn();
    }
    if let Some(lottie) = query_lottie {
        commands.entity(*lottie).despawn();
    }
}
