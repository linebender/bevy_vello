use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    prelude::*,
};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, setup_camera)
    .add_systems(Startup, load_svg);
    embedded_asset!(app, "assets/fountain.svg");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_svg(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    // Spawn SVG with the screen space marker
    commands
        .spawn((
            VelloSvgHandle(asset_server.load("embedded://svg_screenspace/assets/fountain.svg")),
            VelloSvgAnchor::TopLeft,
            VelloRenderSpace::Screen,
        ))
        .insert(Transform::from_xyz(50.0, 50.0, 0.0).with_scale(Vec3::splat(2.0)));
}
