use bevy::{prelude::*, window::PrimaryWindow};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, animate_screenspace)
        .add_systems(PostUpdate, gizmos)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn setup_vector_graphics(mut commands: Commands) {
    commands.spawn((VelloScene2d::new(), Transform::from_scale(Vec3::splat(1.0))));
}

fn animate_screenspace(
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<&Transform, With<Camera2d>>,
    scene_query: Single<(&mut Transform, &mut VelloScene2d), Without<Camera2d>>,
) {
    let window = *window;
    let camera_tf = *camera_query;
    let (mut transform, mut scene) = scene_query.into_inner();

    // Reset scene every frame
    scene.reset();

    // Compute the screen center in world coordinates (camera translation)
    let center_world = Vec2::new(camera_tf.translation.x, camera_tf.translation.y);

    // Radius: choose some fraction of the smaller window dimension (in pixels)
    let radius_pixels = window.width().min(window.height()) * 0.25; // 25% of smaller side

    // Convert pixel-based radius to world units by dividing by camera scale.
    let camera_scale = camera_tf.scale.x; // assume uniform scale on x (Vec3)
    let radius_world = radius_pixels / camera_scale;

    // Animation parameters
    let speed = 2.0; // rotations per second multiplier
    let t = time.elapsed_secs() * speed;
    let x = center_world.x + radius_world * t.cos();
    let y = center_world.y + radius_world * t.sin();

    transform.translation = Vec3::new(x, y, transform.translation.z);

    // Animate color green to blue
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let c = Vec3::lerp(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 1.0), sin_time);

    // Animate the corner radius
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([c.x, c.y, c.z, 1.0]),
        None,
        &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
    );

    // Animate rotation
    transform.rotation = Quat::from_rotation_z(std::f32::consts::TAU * sin_time);
}

fn gizmos(
    mut gizmos: Gizmos,
    scene_tf: Single<&Transform, (With<VelloScene2d>, Without<Camera2d>)>,
) {
    let pos = scene_tf.translation.truncate();

    // Small circle at the position
    gizmos.circle_2d(pos, 5.0, Color::WHITE);

    // Crosshair
    gizmos.line_2d(
        pos + Vec2::new(-10.0, 0.0),
        pos + Vec2::new(10.0, 0.0),
        Color::WHITE,
    );
    gizmos.line_2d(
        pos + Vec2::new(0.0, -10.0),
        pos + Vec2::new(0.0, 10.0),
        Color::WHITE,
    );
}
