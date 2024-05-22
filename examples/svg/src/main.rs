use bevy::asset::{embedded_asset, AssetMetaCheck};
use bevy::prelude::*;
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_systems(Startup, load_svg);
    embedded_asset!(app, "assets/fountain.svg");
    app.run();
}

fn load_svg(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Yes, it's this simple.
    commands.spawn(VelloAssetBundle {
        vector: asset_server.load("embedded://svg/assets/fountain.svg"),
        debug_visualizations: DebugVisualizations::Visible,
        transform: Transform::from_scale(Vec3::splat(5.0)),
        ..default()
    });
}
