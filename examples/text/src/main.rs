use std::f32::consts::PI;

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
    )
    .add_systems(Update, gizmos);
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
        transform: Transform::from_xyz(0.0, -100.0, 0.0)
            .with_rotation(Quat::from_rotation_z(PI / 12.0)),
        ..default()
    });
}

fn setup_screenspace_text(mut commands: Commands) {
    // Bevy text
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
        ))
        .insert(Text::new("Use bevy's Text for UI text!"))
        .insert(TextFont {
            font_size: 24.,
            ..default()
        })
        .insert(TextLayout::new_with_justify(JustifyText::Left));
}

fn gizmos(
    texts: Query<(&VelloTextSection, &GlobalTransform)>,
    assets: Res<Assets<VelloFont>>,
    mut gizmos: Gizmos,
) {
    for (text, gtransform) in texts.iter() {
        let Some(font) = assets.get(text.style.font.id()) else {
            continue;
        };

        let bb_size = font.sizeof(text);

        gizmos.rect_2d(
            Isometry2d::new(
                gtransform.translation().xy(),
                Rot2::radians(gtransform.rotation().to_scaled_axis().z),
            ),
            bb_size * gtransform.scale().xy(),
            Color::WHITE,
        );
    }
}
