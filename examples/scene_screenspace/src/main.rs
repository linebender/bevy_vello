use bevy::prelude::*;
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, animation)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn setup_vector_graphics(mut commands: Commands) {
    // Spawn a scene with the screen space marker
    commands.spawn((VelloScene::new(), VelloRenderSpace::Screen));
}

fn animation(scene: Single<(&mut Transform, &mut VelloScene)>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let (mut transform, mut scene) = scene.into_inner();

    // Reset scene every frame
    scene.reset();

    // Animate color green to blue
    let c = Vec3::lerp(
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        sin_time + 0.5,
    );

    // Animate the corner radius
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([c.x, c.y, c.z, 1.]),
        None,
        &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
    );

    transform.scale = Vec3::lerp(Vec3::ONE * 0.5, Vec3::ONE * 1.0, sin_time);
    transform.translation = Vec3::lerp(Vec3::X * 0.0 + 50.0, Vec3::X * 100.0 + 75.0, sin_time);
    transform.rotation = Quat::from_rotation_z(std::f32::consts::TAU * sin_time);
}
