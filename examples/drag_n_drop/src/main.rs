use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};
use std::ffi::OsStr;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, setup_vector_graphics)
    .add_systems(Update, drag_and_drop);
    embedded_asset!(app, "assets/fountain.svg");
    app.run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn(VelloSvgBundle {
        asset: VelloSvgHandle(asset_server.load("embedded://drag_n_drop/assets/fountain.svg")),
        debug_visualizations: DebugVisualizations::Visible,
        transform: Transform::from_scale(Vec3::splat(5.0)),
        ..default()
    });
}

/// Drag and drop any SVG or Lottie JSON asset into the window and change the
/// displayed asset
fn drag_and_drop(
    mut commands: Commands,
    query_lottie: Option<Single<Entity, With<VelloLottieHandle>>>,
    query_svg: Option<Single<Entity, With<VelloSvgHandle>>>,
    asset_server: ResMut<AssetServer>,
    mut dnd_evr: EventReader<FileDragAndDrop>,
) {
    for ev in dnd_evr.read() {
        if let Some(ref svg) = query_svg {
            commands.entity(**svg).despawn();
        }
        if let Some(ref lottie) = query_lottie {
            commands.entity(**lottie).despawn();
        }
        let FileDragAndDrop::DroppedFile { path_buf, .. } = ev else {
            continue;
        };
        let Some(ext) = path_buf.extension() else {
            continue;
        };
        let svg_ext = OsStr::new("svg");
        let lottie_ext = OsStr::new("json");
        if ext == svg_ext {
            commands.spawn(VelloSvgBundle {
                asset: VelloSvgHandle(asset_server.load(path_buf.clone())),
                ..default()
            });
        } else if ext == lottie_ext {
            commands.spawn(VelloLottieBundle {
                asset: VelloLottieHandle(asset_server.load(path_buf.clone())),
                ..default()
            });
        }
    }
}
