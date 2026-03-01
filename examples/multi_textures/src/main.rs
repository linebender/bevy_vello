//! Demonstrates rendering to multiple independent textures.
//!
//! Two `VelloView` cameras each render different content to separate textures,
//! displayed side-by-side as sprites. `RenderLayers` controls which entities
//! are visible to each camera.

use bevy::{camera::{RenderTarget, visibility::RenderLayers}, prelude::*};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_left, animate_right))
        .run();
}

/// Tag for the left scene (camera A).
#[derive(Component)]
struct LeftScene;

/// Tag for the right scene (camera B).
#[derive(Component)]
struct RightScene;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture_size = UVec2::splat(400);

    // --- Camera A: renders entities on RenderLayers::layer(1) ---
    let image_a = images.add(VelloImage::new(texture_size.x, texture_size.y));
    commands.spawn((
        Camera {
            order: -2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Camera2d,
        VelloView,
        RenderTarget::Image(image_a.clone().into()),
        VelloTargetSize(texture_size),
        VelloClearColor(peniko::Color::new([0.1, 0.1, 0.15, 1.0])),
        RenderLayers::layer(1),
    ));

    // --- Camera B: renders entities on RenderLayers::layer(2) ---
    let image_b = images.add(VelloImage::new(texture_size.x, texture_size.y));
    commands.spawn((
        Camera {
            order: -1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Camera2d,
        VelloView,
        RenderTarget::Image(image_b.clone().into()),
        VelloTargetSize(texture_size),
        VelloClearColor(peniko::Color::new([0.15, 0.1, 0.1, 1.0])),
        RenderLayers::layer(2),
    ));

    // --- Display camera: sees the sprites (default layer 0) ---
    commands.spawn(Camera2d);

    // Display each texture as a sprite, side-by-side
    commands.spawn((
        Sprite {
            image: image_a,
            custom_size: Some(Vec2::splat(400.0)),
            ..default()
        },
        Transform::from_xyz(-220.0, 0.0, 0.0),
    ));
    commands.spawn((
        Sprite {
            image: image_b,
            custom_size: Some(Vec2::splat(400.0)),
            ..default()
        },
        Transform::from_xyz(220.0, 0.0, 0.0),
    ));

    // --- Vello content for Camera A (layer 1) ---
    commands.spawn((VelloScene2d::new(), RenderLayers::layer(1), LeftScene));

    // --- Vello content for Camera B (layer 2) ---
    commands.spawn((VelloScene2d::new(), RenderLayers::layer(2), RightScene));
}

fn animate_left(mut left: Single<&mut VelloScene2d, With<LeftScene>>, time: Res<Time>) {
    let t = time.elapsed_secs();
    let sin_time = t.sin().mul_add(0.5, 0.5);

    // Left scene: animated rounded rectangle (green ↔ blue)
    left.reset();
    let c = Vec3::lerp(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), sin_time);
    left.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([c.x, c.y, c.z, 1.0]),
        None,
        &kurbo::RoundedRect::new(-80.0, -80.0, 80.0, 80.0, (sin_time as f64) * 80.0),
    );
}

fn animate_right(mut right: Single<&mut VelloScene2d, With<RightScene>>, time: Res<Time>) {
    let t = time.elapsed_secs();
    let sin_time = t.sin().mul_add(0.5, 0.5);

    // Right scene: animated circle (red ↔ yellow)
    right.reset();
    let c2 = Vec3::lerp(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.0), sin_time);
    let radius = 40.0 + (sin_time as f64) * 40.0;
    right.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([c2.x, c2.y, c2.z, 1.0]),
        None,
        &kurbo::Circle::new((0.0, 0.0), radius),
    );
}
