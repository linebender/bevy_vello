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
    .add_systems(Startup, enable_debug)
    .add_systems(Startup, setup_camera)
    .add_systems(Startup, load_lottie);
    embedded_asset!(app, "assets/Tiger.json");
    app.run();
}

fn enable_debug(mut options: ResMut<UiDebugOptions>) {
    options.enabled = true;
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_lottie(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let one_third = Val::Percent(100.0 / 3.0);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: one_third,
            top: one_third,
            width: one_third,
            height: one_third,
            ..default()
        },
        UiVelloLottie(asset_server.load("embedded://lottie_ui/assets/Tiger.json")),
    ));
}
