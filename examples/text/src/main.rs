use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    prelude::*,
};
use bevy_vello::{prelude::*, text::VelloTextAnchor, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(
        Startup,
        (setup_camera, setup_screenspace_text, setup_worldspace_text),
    );
    embedded_asset!(app, "assets/Rubik-Medium.ttf");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn setup_worldspace_text(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn(VelloTextSection {
            value: "Default font\nand multi-line support.".to_string(),
            ..default()
        })
        .insert(DebugVisualizations::Visible)
        .insert(VelloTextAnchor::Center);

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
    // Bevy text
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(100.0),
            ..default()
        })
        .insert(Text::new("Use bevy's Text for UI text!"))
        .insert(TextFont {
            font_size: 24.,
            ..default()
        })
        .insert(TextLayout::new_with_justify(JustifyText::Left));
}
