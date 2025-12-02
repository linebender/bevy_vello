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
    .add_systems(Startup, setup)
    .add_systems(Update, drag_and_drop)
    .add_systems(Update, button_system)
    .add_observer(cleanup_scene);
    embedded_asset!(app, "assets/fountain.svg");
    app.run();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2d, bevy_pancam::PanCam::default(), VelloView));
    commands.spawn((
        VelloSvg2d(asset_server.load("embedded://drag_n_drop/assets/fountain.svg")),
        Transform::from_scale(Vec3::splat(5.0)),
    ));
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
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
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

#[expect(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut task_runner: TaskRunner<Option<(String, Vec<u8>)>>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = HOVERED_BUTTON.into();
                border_color.set_all(RED);

                let fut = async move {
                    let handle = rfd::AsyncFileDialog::new().pick_file().await?;
                    let file_name = handle.file_name();
                    let file_contents = String::from_utf8(handle.read().await).ok()?;
                    let file_contents = file_contents.into_bytes();
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
        if file_name.ends_with(".svg") {
            let svg = match bevy_vello::integrations::svg::load_svg_from_bytes(&file) {
                Ok(svg) => svg,
                Err(e) => {
                    tracing::error!("{e:?}");
                    return;
                }
            };
            let handle = asset_server.add(svg);
            commands.trigger(CleanupEvent);
            commands.spawn(VelloSvg2d(handle));
        } else if file_name.ends_with(".json") {
            let lottie = match bevy_vello::integrations::lottie::load_lottie_from_bytes(&file) {
                Ok(lottie) => lottie,
                Err(e) => {
                    tracing::error!("{e:?}");
                    return;
                }
            };
            let handle = asset_server.add(lottie);
            commands.trigger(CleanupEvent);
            commands.spawn(VelloLottieHandle(handle));
        }
    }
}

/// Drag and drop any SVG or Lottie JSON asset into the window and change the
/// displayed asset
fn drag_and_drop(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut dnd_evr: MessageReader<FileDragAndDrop>,
) {
    for ev in dnd_evr.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = ev else {
            continue;
        };
        let Some(ext) = path_buf.extension() else {
            continue;
        };
        let svg_ext = OsStr::new("svg");
        let lottie_ext = OsStr::new("json");
        if ext == svg_ext {
            commands.trigger(CleanupEvent);
            commands.spawn(VelloSvg2d(asset_server.load(path_buf.clone())));
        } else if ext == lottie_ext {
            commands.trigger(CleanupEvent);
            commands.spawn(VelloLottieHandle(asset_server.load(path_buf.clone())));
        }
    }
}

#[derive(Event)]
struct CleanupEvent;

fn cleanup_scene(
    _trigger: On<CleanupEvent>,
    mut commands: Commands,
    query_lottie: Option<Single<Entity, With<VelloLottieHandle>>>,
    query_svg: Option<Single<Entity, With<VelloSvg2d>>>,
) {
    if let Some(svg) = query_svg {
        commands.entity(*svg).despawn();
    }
    if let Some(lottie) = query_lottie {
        commands.entity(*lottie).despawn();
    }
}
