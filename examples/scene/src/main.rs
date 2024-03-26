use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_vello::prelude::*;
use bevy_vello::vello::{kurbo, peniko};

fn main() {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(VelloPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, simple_animation);
    app.run();
}

fn setup_vector_graphics(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), bevy_pancam::PanCam::default()));
    commands.spawn(VelloSceneBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
}

fn simple_animation(
    mut q_transforms: Query<(&mut Transform, &mut VelloScene), With<VelloScene>>,
    time: Res<Time>,
) {
    let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);

    for (mut transform, mut scene) in q_transforms.iter_mut() {
        *scene = VelloScene::default();

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
            peniko::Color::rgb(c.x as f64, c.y as f64, c.z as f64),
            None,
            &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
        );

        transform.scale = Vec3::lerp(Vec3::ONE * 0.5, Vec3::ONE * 1.0, sin_time);
        transform.translation = Vec3::lerp(Vec3::X * -100.0, Vec3::X * 100.0, sin_time);
        transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
    }
}
