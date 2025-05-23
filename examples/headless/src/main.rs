use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            visible: false,
            skip_taskbar: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, (setup, screenshot).chain())
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));

    // Draw a circle
    let mut scene = VelloScene::new();
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([0.0, 1.0, 0.0, 1.0]),
        None,
        &kurbo::Circle::new((0.0, 0.0), 50.0),
    );
    commands.spawn(scene);
}

fn screenshot(mut commands: Commands) {
    let path = "./screenshot.png";
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(exit_system);
}

fn exit_system(_trigger: Trigger<ScreenshotCaptured>, mut exit: EventWriter<AppExit>) {
    exit.write(AppExit::Success);
}
