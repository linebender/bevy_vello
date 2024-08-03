//! Shows how to use render layers.

use bevy::{color::palettes::css, prelude::*, render::view::RenderLayers};
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_systems(Startup, (setup_animation, setup_background))
        .add_systems(
            Update,
            (
                layer0_animation,
                layer1_animation,
                layer2_background,
                run_gizmos,
            ),
        )
        .run();
}

/// A tag that will mark the scene on RenderLayer 0.
#[derive(Component)]
struct Layer0Scene;

/// A tag that will mark the scene on RenderLayer 1.
#[derive(Component)]
struct Layer1Scene;

/// A tag that will mark the scene on RenderLayer 2.
#[derive(Component)]
struct Layer2Scene;

fn setup_animation(mut commands: Commands) {
    const LAYER_0: RenderLayers = RenderLayers::layer(0);
    const LAYER_1: RenderLayers = RenderLayers::layer(1);

    // This camera can see everything on Layer 1 and Layer 2.
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // This camera will render AFTER the blue background camera!
                order: 1,
                ..default()
            },
            ..default()
        },
        LAYER_0.union(&LAYER_1),
    ));

    commands.spawn((VelloSceneBundle::default(), Layer0Scene, LAYER_0));
    commands.spawn((VelloSceneBundle::default(), Layer1Scene, LAYER_1));
}

fn setup_background(mut commands: Commands) {
    const LAYER: RenderLayers = RenderLayers::layer(2);
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // Render first
                order: -1,
                ..default()
            },
            ..default()
        },
        LAYER,
    ));
    commands.spawn((VelloSceneBundle::default(), Layer2Scene, LAYER));
}

fn layer0_animation(
    mut query_scene: Query<(&mut Transform, &mut VelloScene), With<Layer0Scene>>,
    time: Res<Time>,
) {
    let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let (mut transform, mut scene) = query_scene.single_mut();
    // Reset scene every frame
    *scene = VelloScene::default();

    // Animate color green to blue
    let c = Vec3::lerp(
        Vec3::new(0.0, 1.0, -1.0),
        Vec3::new(0.0, 1.0, 1.0),
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
    transform.translation =
        Vec3::lerp(Vec3::X * -100.0, Vec3::X * 100.0, sin_time) - Vec3::Y * 100.0;
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

fn layer1_animation(
    mut query_scene: Query<(&mut Transform, &mut VelloScene), With<Layer1Scene>>,
    time: Res<Time>,
) {
    let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let (mut transform, mut scene) = query_scene.single_mut();
    // Reset scene every frame
    *scene = VelloScene::default();

    // Animate color green to blue
    let c = Vec3::lerp(
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
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
    transform.translation =
        Vec3::lerp(Vec3::X * -100.0, Vec3::X * 100.0, sin_time) * Vec3::NEG_X + Vec3::Y * 100.0;
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

fn layer2_background(mut query_scene: Query<&mut VelloScene, With<Layer2Scene>>) {
    let mut scene = query_scene.single_mut();
    *scene = VelloScene::default();
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::rgb(0.0, 0.0, 1.0),
        None,
        &kurbo::RoundedRect::new(-200.0, -200.0, 200.0, 200.0, 0.0),
    );
}

fn run_gizmos(mut gizmos: Gizmos) {
    gizmos.circle_2d(Vec2::splat(0.0), 20.0, css::RED);
}
