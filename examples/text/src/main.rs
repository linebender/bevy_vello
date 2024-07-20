use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    prelude::*,
};
use bevy_vello::{prelude::*, text::VelloTextAnchor, vello::peniko, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin)
    .add_systems(
        Startup,
        (setup_camera, setup_screenspace_text, setup_worldspace_text),
    );
    embedded_asset!(app, "assets/Rubik-Medium.ttf");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_worldspace_text(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(VelloTextBundle {
        text: VelloTextSection {
            value: "Default font\nand multi-line support.".to_string(),
            ..default()
        },
        text_anchor: VelloTextAnchor::Center,
        transform: Transform::from_xyz(0.0, 100.0, 0.0),
        debug_visualizations: DebugVisualizations::Visible,
        ..default()
    });

    commands.spawn(VelloTextBundle {
        text: VelloTextSection {
            value: "Rubik-Medium Font".to_string(),
            style: VelloTextStyle {
                font: asset_server.load("embedded://text/assets/Rubik-Medium.ttf"),
                font_size: 100.0,
                ..default()
            },
        },
        text_anchor: VelloTextAnchor::Center,
        transform: Transform::from_xyz(0.0, -100.0, 0.0),
        debug_visualizations: DebugVisualizations::Visible,
        ..default()
    });
}

fn setup_screenspace_text(mut commands: Commands) {
    // Vello text
    commands.spawn(VelloTextBundle {
        text: VelloTextSection {
            value: "World-space text rendered by Vello!".to_string(),
            style: VelloTextStyle {
                brush: peniko::Brush::Solid(peniko::Color::RED),
                ..default()
            },
        },
        text_anchor: bevy_vello::text::VelloTextAnchor::TopLeft,
        transform: Transform::from_xyz(100.0, 85.0, 0.0),
        coordinate_space: CoordinateSpace::ScreenSpace,
        debug_visualizations: DebugVisualizations::Visible,
        ..default()
    });

    // Bevy text
    commands.spawn(
        TextBundle::from_section("Screen-space text rendered by Bevy!", TextStyle::default())
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(100.0),
                ..default()
            })
            .with_text_justify(JustifyText::Left),
    );
}
