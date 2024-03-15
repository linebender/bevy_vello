use bevy::asset::{embedded_asset, AssetMetaCheck};
use bevy::prelude::*;
use bevy_vello::prelude::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_screenspace_vectors,
                setup_worldspace_vectors,
            ),
        );
    embedded_asset!(app, "src", "squid.json");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), bevy_pancam::PanCam::default()));
}

fn setup_worldspace_vectors(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    const AMOUNT: i32 = 5;
    const SPACING: f32 = 50.0;
    const SIZE: f32 = 0.1;

    // Top Right
    for i in 1..=AMOUNT {
        commands
            .spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/squid.json"),
                transform: Transform::from_scale(Vec3::splat(SIZE))
                    .with_translation(Vec3::splat(i as f32 * SPACING)),
                debug_visualizations: DebugVisualizations::Visible,
                coordinate_space: CoordinateSpace::WorldSpace,
                z_function: ZFunction::BbTop,
                ..default()
            })
            .insert(
                Theme::new()
                    .add("suckers ", Color::RED)
                    .add("suckers Flip", Color::RED),
            );
    }

    // Bottom Left
    for i in 1..=AMOUNT {
        commands
            .spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/squid.json"),
                transform: Transform::from_scale(Vec3::splat(SIZE))
                    .with_translation(Vec3::splat(-i as f32 * SPACING)),
                debug_visualizations: DebugVisualizations::Visible,
                coordinate_space: CoordinateSpace::WorldSpace,
                z_function: ZFunction::BbRight,
                ..default()
            })
            .insert(
                Theme::new()
                    .add("suckers ", Color::GREEN)
                    .add("suckers Flip", Color::GREEN),
            );
    }

    // Top Left
    for i in 1..=AMOUNT {
        commands
            .spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/squid.json"),
                transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(
                    (Vec3::X * Vec3::splat(-i as f32)) * SPACING
                        + (Vec3::Y * Vec3::splat(i as f32)) * SPACING,
                ),
                debug_visualizations: DebugVisualizations::Visible,
                coordinate_space: CoordinateSpace::WorldSpace,
                z_function: ZFunction::BbLeft,
                ..default()
            })
            .insert(
                Theme::new()
                    .add("suckers ", Color::YELLOW)
                    .add("suckers Flip", Color::YELLOW),
            );
    }

    // Bottom right
    for i in 1..=AMOUNT {
        commands
            .spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/squid.json"),
                transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(
                    (Vec3::X * Vec3::splat(i as f32)) * SPACING
                        + (Vec3::Y * Vec3::splat(-i as f32)) * SPACING,
                ),
                debug_visualizations: DebugVisualizations::Visible,
                coordinate_space: CoordinateSpace::WorldSpace,
                z_function: ZFunction::BbBottom,
                ..default()
            })
            .insert(
                Theme::new()
                    .add("suckers ", Color::BLUE)
                    .add("suckers Flip", Color::BLUE),
            );
    }
}

fn setup_screenspace_vectors(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    for i in 1..=10 {
        commands.spawn(VelloAssetBundle {
            vector: asset_server.load("embedded://z_ordering/squid.json"),
            transform: Transform::from_scale(Vec3::splat(0.03))
                .with_translation(Vec3::splat(i as f32 * 20.0)),
            debug_visualizations: DebugVisualizations::Visible,
            coordinate_space: CoordinateSpace::ScreenSpace,
            z_function: ZFunction::BbRight,
            ..default()
        });
    }
}
