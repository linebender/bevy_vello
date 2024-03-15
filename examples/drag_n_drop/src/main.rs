use bevy::asset::{embedded_asset, AssetMetaCheck};
use bevy::prelude::*;
use bevy_vello::prelude::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, drag_and_drop);
    embedded_asset!(app, "src", "fountain.svg");
    app.run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(VelloAssetBundle {
        vector: asset_server.load::<VelloAsset>("embedded://drag_n_drop/fountain.svg"),
        debug_visualizations: DebugVisualizations::Visible,
        transform: Transform::from_scale(Vec3::splat(5.0)),
        ..default()
    });
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
