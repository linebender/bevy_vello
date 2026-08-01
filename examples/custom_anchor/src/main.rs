use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    prelude::*,
};
use bevy_vello::{VelloPlugin, prelude::*};

const SVG_PATH: &str = "embedded://custom_anchor/assets/Ghostscript_Tiger.svg";
const LABEL_Y: f32 = -260.0;
const MARKER_SIZE: f32 = 56.0;
const TIGER_SCALE: f32 = 0.32;
const CUSTOM_OFFSET: Vec2 = Vec2::new(90.0, 70.0);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.11)))
    .add_systems(Startup, (setup_camera, spawn_scene))
    .add_systems(Update, rotate_anchors);
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn spawn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    let svg = asset_server.load(SVG_PATH);
    let title_brush =
        vello::peniko::Brush::Solid(vello::peniko::Color::from_rgb8(0xFA, 0xEB, 0xD7));

    commands.spawn((
        VelloText2d {
            value: "Custom anchor offset demo".to_string(),
            style: VelloTextStyle {
                font_size: 30.0,
                brush: title_brush,
                ..default()
            },
            ..default()
        },
        VelloAnchor::Center,
        Transform::from_xyz(0.0, 300.0, 1.0),
    ));

    spawn_anchor_demo(
        &mut commands,
        svg.clone(),
        Vec3::new(-260.0, 30.0, 0.0),
        VelloAnchor::Center,
        "Center anchor",
    );
    spawn_anchor_demo(
        &mut commands,
        svg,
        Vec3::new(260.0, 30.0, 0.0),
        VelloAnchor::CenterOffset(CUSTOM_OFFSET),
        &format!("Custom offset ({}, {})", CUSTOM_OFFSET.x, CUSTOM_OFFSET.y),
    );
}

fn spawn_anchor_demo(
    commands: &mut Commands,
    svg: Handle<VelloSvg>,
    translation: Vec3,
    anchor: VelloAnchor,
    label: &str,
) {
    let marker_brush =
        vello::peniko::Brush::Solid(vello::peniko::Color::from_rgb8(0xFF, 0x45, 0x00));
    let label_brush = vello::peniko::Brush::Solid(vello::peniko::Color::WHITE);

    commands.spawn((
        VelloText2d {
            value: "+".to_string(),
            style: VelloTextStyle {
                font_size: MARKER_SIZE,
                brush: marker_brush,
                ..default()
            },
            ..default()
        },
        VelloAnchor::Center,
        Transform::from_translation(translation + Vec3::new(0.0, 0.0, 2.0)),
    ));

    commands.spawn((
        VelloSvg2d(svg),
        anchor,
        Transform::from_translation(translation).with_scale(Vec3::splat(TIGER_SCALE)),
        Rotates,
    ));

    commands.spawn((
        VelloText2d {
            value: label.to_string(),
            style: VelloTextStyle {
                font_size: 22.0,
                brush: label_brush,
                ..default()
            },
            ..default()
        },
        VelloAnchor::Center,
        Transform::from_translation(translation + Vec3::new(0.0, LABEL_Y, 1.0)),
    ));
}

#[derive(Component)]
struct Rotates;

fn rotate_anchors(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_rotation_z(time.elapsed_secs() * 0.8);
    }
}
