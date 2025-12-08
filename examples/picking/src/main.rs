use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    color::palettes::css,
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
    .add_systems(Startup, enable_debug)
    .add_systems(Startup, load_lottie)
    .add_systems(Update, print_metadata);
    embedded_asset!(app, "assets/calendar.json");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn enable_debug(mut config: ResMut<GizmoConfigStore>) {
    config.config_mut::<AabbGizmoConfigGroup>().1.draw_all = true;
}

fn load_lottie(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn((
            VelloLottie2d(asset_server.load("embedded://picking/assets/calendar.json")),
            Theme::new().add("calendar", css::YELLOW.into()),
            PlaybackOptions {
                speed: 0.0,
                ..Default::default()
            },
            Transform::from_scale(Vec3::splat(20.0)),
        ))
        .observe(on_pointer_press)
        .observe(on_pointer_release)
        .observe(on_pointer_enter)
        .observe(on_pointer_leave);
}

fn on_pointer_enter(trigger: On<Pointer<Over>>, mut theme: Single<&mut Theme>) {
    println!("Mouse entered: {}", trigger.event());
    *(theme.as_mut()) = Theme::new().add("calendar", css::ORANGE.into());
}

fn on_pointer_press(trigger: On<Pointer<Press>>, mut theme: Single<&mut Theme>) {
    println!("Mouse pressed: {}", trigger.event());
    *(theme.as_mut()) = Theme::new().add("calendar", css::RED.into());
}

fn on_pointer_release(trigger: On<Pointer<Release>>, mut theme: Single<&mut Theme>) {
    println!("Mouse released: {}", trigger.event());
    *(theme.as_mut()) = Theme::new().add("calendar", css::ORANGE.into());
}

fn on_pointer_leave(trigger: On<Pointer<Out>>, mut theme: Single<&mut Theme>) {
    println!("Mouse exited ({})", trigger.event());
    *(theme.as_mut()) = Theme::new().add("calendar", css::YELLOW.into());
}

fn print_metadata(
    mut asset_ev: MessageReader<AssetEvent<VelloLottie>>,
    assets: Res<Assets<VelloLottie>>,
) {
    for ev in asset_ev.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let asset = assets.get(*id).unwrap();
            println!(
                "Lottie layers in composition:\n{:#?}",
                asset.composition.as_ref().get_layers().collect::<Vec<_>>()
            );
        }
    }
}
