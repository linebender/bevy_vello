use bevy::asset::{embedded_asset, AssetMetaCheck};
use bevy::prelude::*;
use bevy_vello::vello::peniko::{Brush, Color};
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin)
    .add_systems(
        Startup,
        (
            setup_camera,
            setup_screenspace_vectors,
            setup_worldspace_vectors,
        ),
    );
    embedded_asset!(app, "assets/google_fonts/squid.json");
    embedded_asset!(app, "assets/fonts/Rubik-Medium.ttf");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_worldspace_vectors(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    const AMOUNT: i32 = 5;
    const X_SPACING: f32 = 220.0;
    const Y_SPACING: f32 = 15.0;
    const SIZE: f32 = 0.05;
    const OFFSET: f32 = 102.4;

    // Show assets
    let mut row = |label: &str, y: f32, zfn: ZFunction| {
        commands.spawn(VelloTextBundle {
            font: asset_server.load("embedded://z_ordering/assets/fonts/Rubik-Medium.ttf"),
            alignment: VelloTextAlignment::Right,
            coordinate_space: CoordinateSpace::WorldSpace,
            text: VelloText {
                content: label.into(),
                brush: Some(Brush::Solid(Color::WHITE)),
                size: 50.0 / SIZE,
            },
            transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(Vec3::new(
                -10.0 / SIZE,
                y * Y_SPACING,
                f32::MAX,
            )),
            ..default()
        });

        commands.spawn(VelloTextBundle {
            font: asset_server.load("embedded://z_ordering/assets/fonts/Rubik-Medium.ttf"),
            alignment: VelloTextAlignment::Bottom,
            coordinate_space: CoordinateSpace::WorldSpace,
            text: VelloText {
                content: "Center".to_string(),
                brush: Some(Brush::Solid(Color::WHITE)),
                size: 50.0 / SIZE,
            },
            transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(Vec3::new(
                0.0,
                10.0 / SIZE,
                f32::MAX,
            )),
            ..default()
        });
        commands.spawn(VelloTextBundle {
            font: asset_server.load("embedded://z_ordering/assets/fonts/Rubik-Medium.ttf"),
            alignment: VelloTextAlignment::Bottom,
            coordinate_space: CoordinateSpace::WorldSpace,
            text: VelloText {
                content: "Bottom".to_string(),
                brush: Some(Brush::Solid(Color::WHITE)),
                size: 50.0 / SIZE,
            },
            transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(Vec3::new(
                X_SPACING,
                10.0 / SIZE,
                f32::MAX,
            )),
            ..default()
        });
        commands.spawn(VelloTextBundle {
            font: asset_server.load("embedded://z_ordering/assets/fonts/Rubik-Medium.ttf"),
            alignment: VelloTextAlignment::Bottom,
            coordinate_space: CoordinateSpace::WorldSpace,
            text: VelloText {
                content: "Top".to_string(),
                brush: Some(Brush::Solid(Color::WHITE)),
                size: 50.0 / SIZE,
            },
            transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(Vec3::new(
                X_SPACING * 2.0,
                10.0 / SIZE,
                f32::MAX,
            )),
            ..default()
        });
        commands.spawn(VelloTextBundle {
            font: asset_server.load("embedded://z_ordering/assets/fonts/Rubik-Medium.ttf"),
            alignment: VelloTextAlignment::Bottom,
            coordinate_space: CoordinateSpace::WorldSpace,
            text: VelloText {
                content: "Right".to_string(),
                brush: Some(Brush::Solid(Color::WHITE)),
                size: 50.0 / SIZE,
            },
            transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(Vec3::new(
                X_SPACING * 3.0,
                10.0 / SIZE,
                f32::MAX,
            )),
            ..default()
        });
        commands.spawn(VelloTextBundle {
            font: asset_server.load("embedded://z_ordering/assets/fonts/Rubik-Medium.ttf"),
            alignment: VelloTextAlignment::Bottom,
            coordinate_space: CoordinateSpace::WorldSpace,
            text: VelloText {
                content: "Left".to_string(),
                brush: Some(Brush::Solid(Color::WHITE)),
                size: 50.0 / SIZE,
            },
            transform: Transform::from_scale(Vec3::splat(SIZE)).with_translation(Vec3::new(
                X_SPACING * 4.0,
                10.0 / SIZE,
                f32::MAX,
            )),
            ..default()
        });

        for i in (1..AMOUNT).rev() {
            // Assets
            commands.spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/assets/google_fonts/squid.json"),
                alignment: VelloAssetAlignment::Center,
                transform: Transform::from_scale(Vec3::splat(i as f32 * SIZE))
                    .with_translation(Vec3::new(0.0, y * Y_SPACING, 0.0)),
                debug_visualizations: DebugVisualizations::Visible,
                z_function: zfn,
                ..default()
            });

            commands.spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/assets/google_fonts/squid.json"),
                alignment: VelloAssetAlignment::Bottom,
                transform: Transform::from_translation(Vec3::new(
                    X_SPACING,
                    y * Y_SPACING - OFFSET,
                    0.0,
                ))
                .with_scale(Vec3::splat(i as f32 * SIZE)),
                debug_visualizations: DebugVisualizations::Visible,
                z_function: zfn,
                ..default()
            });

            commands.spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/assets/google_fonts/squid.json"),
                alignment: VelloAssetAlignment::Top,
                transform: Transform::from_translation(Vec3::new(
                    X_SPACING * 2.0,
                    y * Y_SPACING + OFFSET,
                    0.0,
                ))
                .with_scale(Vec3::splat(i as f32 * SIZE)),
                debug_visualizations: DebugVisualizations::Visible,
                z_function: zfn,
                ..default()
            });
            commands.spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/assets/google_fonts/squid.json"),
                alignment: VelloAssetAlignment::Right,
                transform: Transform::from_translation(Vec3::new(
                    X_SPACING * 3.0 + OFFSET,
                    y * Y_SPACING,
                    0.0,
                ))
                .with_scale(Vec3::splat(i as f32 * SIZE)),
                debug_visualizations: DebugVisualizations::Visible,
                z_function: zfn,
                ..default()
            });
            commands.spawn(VelloAssetBundle {
                vector: asset_server.load("embedded://z_ordering/assets/google_fonts/squid.json"),
                alignment: VelloAssetAlignment::Left,
                transform: Transform::from_translation(Vec3::new(
                    X_SPACING * 4.0 - OFFSET,
                    y * Y_SPACING,
                    0.0,
                ))
                .with_scale(Vec3::splat(i as f32 * SIZE)),
                debug_visualizations: DebugVisualizations::Visible,
                z_function: zfn,
                ..default()
            });
        }
    };

    for (i, (label, zfn)) in [
        ("ZDefault", ZFunction::default()),
        ("BbBottom", ZFunction::BbBottom),
        ("BbRight", ZFunction::BbRight),
        ("BbLeft", ZFunction::BbLeft),
        ("BbTop", ZFunction::BbTop),
        ("BbBottomInverse", ZFunction::BbBottomInverse),
        ("BbRightInverse", ZFunction::BbRightInverse),
        ("BbLeftInverse", ZFunction::BbLeftInverse),
        ("BbTopInverse", ZFunction::BbTopInverse),
    ]
    .iter()
    .enumerate()
    {
        row(label, -(i as f32) * Y_SPACING, *zfn);
    }
}

fn setup_screenspace_vectors(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(VelloAssetBundle {
        vector: asset_server.load("embedded://z_ordering/assets/google_fonts/squid.json"),
        transform: Transform::from_scale(Vec3::splat(0.03)).with_translation(Vec3::splat(20.0)),
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..default()
    });
}
