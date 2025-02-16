use bevy::{diagnostic::DiagnosticsStore, prelude::*};
use bevy_vello::{diagnostics::VelloDiagnosticsPlugin, prelude::*, VelloPlugin};
use std::ops::DerefMut;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_plugins(VelloDiagnosticsPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, simple_animation)
        .add_systems(Update, print_diagnostics);

    app.run();
}

fn setup_vector_graphics(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
    commands.spawn(VelloScene::new());
}

fn simple_animation(mut query_scene: Single<(&mut Transform, &mut VelloScene)>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let (ref mut transform, ref mut scene) = query_scene.deref_mut();
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
    transform.translation = Vec3::lerp(Vec3::X * -100.0, Vec3::X * 100.0, sin_time);
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

fn print_diagnostics(diagnostics: Res<DiagnosticsStore>) {
    let diagnostic = diagnostics
        .get(&VelloDiagnosticsPlugin::SCENE_COUNT)
        .unwrap();

    if let Some(scene_count) = diagnostic.measurement() {
        info!("Total scenes: {}", scene_count.value);
    }
}
