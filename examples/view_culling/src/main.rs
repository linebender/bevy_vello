use std::ops::DerefMut;

use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    camera::primitives::Aabb,
    diagnostic::DiagnosticsStore,
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
    .add_systems(Startup, load_view_culling)
    .add_systems(
        Update,
        (
            left_right,
            up_down,
            right_left,
            down_up,
            simple_animation,
            log_visibility,
        )
            .chain(),
    );

    embedded_asset!(app, "assets/Tiger.json");
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");

    app.run();
}

fn log_visibility(
    diagnostics: Res<DiagnosticsStore>,
    scene: Single<&ViewVisibility, With<VelloScene2d>>,
    lottie: Single<&ViewVisibility, With<VelloLottie2d>>,
    svg: Single<&ViewVisibility, With<VelloSvg2d>>,
    text: Single<&ViewVisibility, With<VelloText2d>>,
) {
    let visible_status = format!(
        r#"
{{
    visibility: {{
        scene: {},
        lottie: {},
        svg: {},
        text: {}
    }},
    render: {{
        paths: {},
        path_segs: {}
    }}
}}"#,
        scene.get(),
        lottie.get(),
        svg.get(),
        text.get(),
        diagnostics
            .get(&bevy_vello::render::diagnostics::PATH_COUNT)
            .and_then(|d| d.measurement())
            .map(|m| m.value)
            .unwrap_or(0.0),
        diagnostics
            .get(&bevy_vello::render::diagnostics::PATH_SEGMENTS_COUNT)
            .and_then(|d| d.measurement())
            .map(|m| m.value)
            .unwrap_or(0.0)
    );

    println!("{visible_status}");
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn enable_debug(mut config: ResMut<GizmoConfigStore>) {
    config.config_mut::<AabbGizmoConfigGroup>().1.draw_all = true;
}

fn load_view_culling(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn(VelloScene2d::new())
        .insert(RightLeft)
        // For scene culling, you must specify the bounding box yourself!
        .insert(Aabb::from_min_max(
            Vec3::new(-50.0, -50.0, 0.0),
            Vec3::new(50.0, 50.0, 0.0),
        ));

    commands
        .spawn(VelloLottie2d(
            asset_server.load("embedded://view_culling/assets/Tiger.json"),
        ))
        .insert(Transform::from_scale(Vec3::splat(0.2)))
        .insert(LeftRight);

    commands
        .spawn(VelloSvg2d(
            asset_server.load("embedded://view_culling/assets/Ghostscript_Tiger.svg"),
        ))
        .insert(Transform::from_scale(Vec3::splat(0.2)))
        .insert(DownUp);

    commands
        .spawn((
            VelloText2d {
                value: "View culled text".to_string(),
                style: VelloTextStyle {
                    font_size: 24.0,
                    ..default()
                },
                ..default()
            },
            VelloTextAnchor::Center,
        ))
        .insert(UpDown);
}

fn simple_animation(mut query_scene: Single<(&mut Transform, &mut VelloScene2d)>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let (transform, scene) = query_scene.deref_mut();
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
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

#[derive(Component)]
struct LeftRight;

const ANIMATION_SPEED: f32 = 0.5;

fn left_right(mut query: Query<&mut Transform, With<LeftRight>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::X * -800.0, Vec3::X * 800.0, sin_time);
    }
}

#[derive(Component)]
struct RightLeft;

fn right_left(mut query: Query<&mut Transform, With<RightLeft>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::X * 800.0, Vec3::X * -800.0, sin_time);
    }
}

#[derive(Component)]
struct UpDown;

fn up_down(mut query: Query<&mut Transform, With<UpDown>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::Y * -500.0, Vec3::Y * 500.0, sin_time);
    }
}

#[derive(Component)]
struct DownUp;

fn down_up(mut query: Query<&mut Transform, With<DownUp>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::Y * 500.0, Vec3::Y * -500.0, sin_time);
    }
}
