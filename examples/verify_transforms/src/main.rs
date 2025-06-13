use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    color::palettes::css,
    prelude::*,
    ui::ContentSize,
};
use bevy_vello::{
    VelloPlugin,
    prelude::*,
    render::{VelloScreenScale, VelloWorldScale},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, spawn_all_the_things)
    // .insert_resource(VelloScreenScale(2.0))
    // .insert_resource(VelloWorldScale(2.0))
    .add_systems(Update, (rotate, gizmos));
    embedded_asset!(app, "assets/fountain.svg");
    app.run();
}

const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEIGHT: f32 = 720.0;

fn spawn_all_the_things(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2d, VelloView));

    // Svg in world
    commands
        .spawn((
            VelloSvgHandle(asset_server.load("embedded://verify_transforms/assets/fountain.svg")),
            Transform::from_xyz(0.0, SCREEN_HEIGHT / 4., 0.0),
            RotateThing,
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloTextSection {
                    value: "SVG in world space".to_string(),
                    text_align: VelloTextAlign::Middle,
                    ..default()
                },
                VelloTextAnchor::Center,
                Transform::from_xyz(0.0, -50., 0.0),
            ));
        });

    // Svg in Bevy UI
    commands
        .spawn(Node {
            position_type: PositionType::Relative,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(SCREEN_HEIGHT / 4.),
                        left: Val::Px(SCREEN_WIDTH / 4.),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    VelloSvgHandle(
                        asset_server.load("embedded://verify_transforms/assets/fountain.svg"),
                    ),
                    RotateThing,
                    BorderColor(css::FUCHSIA.with_alpha(0.5).into()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            top: Val::Px(50.0),
                            ..default()
                        },
                        ContentSize::default(),
                        VelloTextSection {
                            value: "SVG in bevy_ui".to_string(),
                            text_align: VelloTextAlign::Middle,
                            ..default()
                        },
                        VelloTextAnchor::Center,
                    ));
                });
        });

    // Svg in Screen Space
    commands
        .spawn((
            VelloScreenSpace,
            VelloSvgHandle(asset_server.load("embedded://verify_transforms/assets/fountain.svg")),
            Transform::from_xyz(SCREEN_WIDTH / 4. * 3., SCREEN_HEIGHT / 4., 0.0),
            RotateThing,
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloScreenSpace,
                VelloTextSection {
                    value: "SVG in screen space".to_string(),
                    text_align: VelloTextAlign::Middle,
                    ..default()
                },
                VelloTextAnchor::Center,
                Transform::from_xyz(0.0, 50.0, 0.0),
            ));
        });
}

#[derive(Component, Clone)]
pub struct RotateThing;

fn rotate(mut rotate_q: Query<&mut Transform, With<RotateThing>>, time: Res<Time>) {
    for mut transform in rotate_q.iter_mut() {
        transform.rotate_z(-0.5 * time.delta_secs());
    }
}

#[allow(clippy::type_complexity)]
fn gizmos(
    svg: Query<(&VelloSvgHandle, &GlobalTransform), (Without<Node>, Without<VelloScreenSpace>)>,
    assets: Res<Assets<VelloSvg>>,
    mut gizmos: Gizmos,
) {
    for (svg, gtransform) in svg.iter() {
        let Some(svg) = assets.get(svg.id()) else {
            continue;
        };

        gizmos.rect_2d(
            Isometry2d::new(
                gtransform.translation().xy(),
                Rot2::radians(gtransform.rotation().to_scaled_axis().z),
            ),
            Vec2::new(svg.width, svg.height) * gtransform.scale().xy(),
            Color::WHITE,
        );
    }
}
