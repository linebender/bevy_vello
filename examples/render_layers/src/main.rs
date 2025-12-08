//! Shows how to use render layers.

use std::ops::DerefMut;

use bevy::{
    camera::{primitives::Aabb, visibility::RenderLayers},
    color::palettes::css,
    prelude::*,
};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin {
            canvas_render_layers: RenderLayers::layer(1).with(2),
            ..default()
        })
        .add_systems(Startup, (setup_gizmos, setup_scene))
        .add_systems(Update, (animation, run_gizmos))
        .run();
}

/// A tag that will mark the scene with animation.
#[derive(Component)]
struct AnimationScene;

/// A tag that will mark the scene with the blue square.
#[derive(Component)]
struct BackgroundScene;

fn setup_gizmos(mut commands: Commands, mut config_store: ResMut<GizmoConfigStore>) {
    // This camera can only see Gizmos.
    commands.spawn((
        Camera2d,
        Camera {
            // This camera will render LAST.
            order: 1,
            // Ensure the gizmos render over Vello without a clear color
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(3),
    ));
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.render_layers = RenderLayers::layer(3);
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // This camera will render first.
            order: -1,
            ..default()
        },
        RenderLayers::layer(1).with(2),
        VelloView,
    ));

    commands.spawn((
        {
            let mut scene = VelloScene2d::new();
            scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::default(),
                peniko::Color::new([0.0, 0.0, 1.0, 1.0]),
                None,
                &kurbo::RoundedRect::new(-200.0, -200.0, 200.0, 200.0, 0.0),
            );
            scene
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        Aabb::from_min_max(Vec3::new(-200.0, -200.0, 0.0), Vec3::new(200.0, 20.0, 0.0)),
        BackgroundScene,
        RenderLayers::layer(1),
    ));
    commands.spawn((
        VelloScene2d::new(),
        Transform::from_xyz(0.0, 0.0, 5.0),
        Aabb::from_min_max(Vec3::new(-50.0, -50.0, 0.0), Vec3::new(50.0, 50.0, 0.0)),
        AnimationScene,
        RenderLayers::layer(2),
    ));
}

fn animation(
    mut query_scene: Single<(&mut Transform, &mut VelloScene2d), With<AnimationScene>>,
    time: Res<Time>,
) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let (transform, scene) = query_scene.deref_mut();
    // Reset scene every frame
    scene.reset();

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
        peniko::Color::new([c.x, c.y, c.z, 1.]),
        None,
        &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
    );

    transform.scale = Vec3::lerp(Vec3::ONE * 0.5, Vec3::ONE * 1.0, sin_time);
    transform.translation = Vec3::lerp(Vec3::X * -100.0, Vec3::X * 100.0, sin_time).with_z(0.0);
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

fn run_gizmos(mut gizmos: Gizmos) {
    gizmos.circle_2d(Vec2::splat(0.0), 100.0, css::MAGENTA);
}
