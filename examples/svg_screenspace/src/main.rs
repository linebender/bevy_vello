use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    prelude::*,
    window::PrimaryWindow,
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
    .add_systems(Startup, load_svg)
    .add_systems(Update, animate_svg_worldspace);
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_svg(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        VelloSvg2d(asset_server.load("embedded://svg_screenspace/assets/Ghostscript_Tiger.svg")),
        VelloSvgAnchor::Center,
        Transform::from_scale(Vec3::splat(0.25)),
    ));
}

fn animate_svg_worldspace(
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<&Transform, With<Camera2d>>,
    mut transform: Single<&mut Transform, (With<VelloSvg2d>, Without<Camera2d>)>,
) -> Result {
    let window = *window;
    let camera_tf = *camera_query;

    // Compute the screen center in world coordinates (camera translation)
    let center_world = Vec2::new(camera_tf.translation.x, camera_tf.translation.y);

    // Radius: choose some fraction of the smaller window dimension (in pixels)
    let radius_pixels = window.width().min(window.height()) * 0.25; // 25% of smaller side

    // Convert pixel-based radius to world units by dividing by camera scale.
    // For Camera2d, zooming is usually represented by camera_tf.scale (uniform).
    // If your camera uses other projection settings, you may need to adjust conversion.
    let camera_scale = camera_tf.scale.x; // assume uniform scale on x (Vec3)
    let radius_world = radius_pixels / camera_scale;

    // Animation parameters
    let speed = 2.0; // rotations per second multiplier
    let t = time.elapsed_secs() * speed;

    let x = center_world.x + radius_world * t.cos();
    let y = center_world.y + radius_world * t.sin();

    transform.translation = Vec3::new(x, y, transform.translation.z);

    Ok(())
}
