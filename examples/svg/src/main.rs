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
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_svg(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn((
            VelloSvg2d(asset_server.load("embedded://svg/assets/Ghostscript_Tiger.svg")),
            VelloSvgAnchor::Center,
        ))
        .insert(Transform::from_scale(Vec3::splat(0.5)));
}
