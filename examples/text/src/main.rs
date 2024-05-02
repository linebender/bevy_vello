use bevy::asset::{embedded_asset, AssetMetaCheck};
use bevy::prelude::*;
use bevy_vello::text::VelloTextAlignment;
use bevy_vello::vello::peniko;
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(
            Startup,
            (setup_camera, setup_screenspace_text, setup_worldspace_text),
        );
    embedded_asset!(app, "assets/Rubik-Medium.ttf");
    embedded_asset!(app, "assets/Rubik-Medium.vttf");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), bevy_pancam::PanCam::default()));
}

fn setup_worldspace_text(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(VelloTextBundle {
        font: asset_server.load("embedded://text/assets/Rubik-Medium.vttf"),
        text: VelloText {
            content: "This text is centered\non x and y axes".to_string(),
            size: 50.0,
            brush: None,
        },
        alignment: VelloTextAlignment::Center,
        transform: Transform::from_xyz(100.0, 100.0, 0.0),
        debug_visualizations: DebugVisualizations::Visible,
        ..default()
    });

    commands.spawn(VelloTextBundle {
        font: asset_server.load("embedded://text/assets/Rubik-Medium.vttf"),
        text: VelloText {
            content: "WXYZ".to_string(),
            size: 100.0,
            brush: None,
        },
        transform: Transform::from_xyz(-100.0, -100.0, 0.0),
        debug_visualizations: DebugVisualizations::Visible,
        ..default()
    });
}

fn setup_screenspace_text(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    // Vello text
    commands.spawn(VelloTextBundle {
        font: asset_server.load("embedded://text/assets/Rubik-Medium.vttf"),
        text: VelloText {
            content: "Text rendered by Vello!".to_string(),
            size: 15.0,
            brush: Some(peniko::Brush::Solid(peniko::Color::RED)),
        },
        alignment: bevy_vello::text::VelloTextAlignment::TopLeft,
        transform: Transform::from_xyz(100.0, 85.0, 0.0),
        coordinate_space: CoordinateSpace::ScreenSpace,
        debug_visualizations: DebugVisualizations::Visible,
        ..default()
    });

    // Bevy text (probably the better API)
    commands.spawn(
        TextBundle::from_section(
            "Text rendered by Bevy!",
            TextStyle {
                font: asset_server.load("embedded://text/assets/Rubik-Medium.ttf"),
                font_size: 15.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(100.0),
            ..default()
        })
        .with_text_justify(JustifyText::Left),
    );
}
