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
    .add_systems(Startup, load_lottie);
    embedded_asset!(app, "assets/Tiger.json");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_lottie(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    // You can also use `VelloLottieBundle`
    commands
        .spawn((
            VelloLottieHandle(asset_server.load("embedded://lottie/assets/Tiger.json")),
            VelloLottieAnchor::Center,
            VelloRenderSpace::Screen,
        ))
        .insert(Transform::from_xyz(0.0, 50.0, 0.0).with_scale(Vec3::splat(2.0)));
}
