use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use bevy_vello::{VelloPlugin, prelude::*};

/// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

/// Tag for the animated Vello scene.
#[derive(Component)]
struct AnimatedScene;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(VelloPlugin {
            use_cpu: false,
            antialiasing: vello::AaConfig::Msaa8,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (cube_rotator_system, animate_scene))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // --- Vello camera: renders to a 512x512 texture on RenderLayers::layer(1) ---
    let texture = images.add(VelloImage::new(512, 512));
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        VelloView,
        RenderTarget::Image(texture.clone().into()),
        VelloClearColor(peniko::Color::WHITE),
        RenderLayers::layer(1),
    ));

    // --- Animated Vello scene (visible to the Vello camera) ---
    commands.spawn((VelloScene2d::new(), RenderLayers::layer(1), AnimatedScene));

    // --- 3D scene ---
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 4.0))),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, 1.5)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 5.0)),
        MainPassCube,
    ));

    commands
        .spawn(PointLight::default())
        .insert(Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)));

    commands
        .spawn((Camera3d::default(), FreeCamera::default()))
        .insert(Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y));
}

fn animate_scene(mut scene: Single<&mut VelloScene2d, With<AnimatedScene>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let c = Vec3::lerp(
        Vec3::new(-1.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 1.0),
        sin_time + 0.5,
    );

    scene.reset();
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([c.x, c.y, c.z, 1.0]),
        None,
        &kurbo::RoundedRect::new(-128.0, -128.0, 128.0, 128.0, (sin_time as f64) * 128.0),
    );
}

/// Rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.0 * time.delta_secs());
        transform.rotate_y(0.7 * time.delta_secs());
    }
}
