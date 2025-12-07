use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    camera::primitives::Aabb,
    color::palettes::css,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    ui::ContentSize,
    window::WindowResolution,
};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(VelloPlugin::default())
    .add_systems(
        Startup,
        (
            spawn_camera,
            enable_debug,
            spawn_bevy_ui,
            spawn_scenes,
            spawn_instructions,
        ),
    )
    .add_systems(
        Update,
        (
            rotate,
            simple_ui_animation,
            simple_non_ui_animation,
            scale_control,
        ),
    );
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    embedded_asset!(app, "assets/Tiger.json");
    app.run();
}

const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEIGHT: f32 = 720.0;
const CELL_WIDTH: f32 = SCREEN_WIDTH / 4.0;
const CELL_HEIGHT: f32 = SCREEN_HEIGHT / 5.0;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn enable_debug(mut options: ResMut<UiDebugOptions>, mut config: ResMut<GizmoConfigStore>) {
    options.enabled = true;
    config.config_mut::<AabbGizmoConfigGroup>().1.draw_all = true;
}

fn spawn_instructions(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        UiVelloText {
            value: "Press 1 to scale down, press 2 to scale up, press 3 to reset scale to 1.0"
                .to_string(),
            text_align: VelloTextAlign::Middle,
            style: VelloTextStyle {
                font_size: 16.,
                ..default()
            },
            ..default()
        },
        VelloTextAnchor::Center,
    ));
}

fn spawn_bevy_ui(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn(Node {
            position_type: PositionType::Relative,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(50.0),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            ..default()
                        },
                        UiVelloScene::new(),
                    ));

                    parent.spawn((
                        ContentSize::default(),
                        Node::default(),
                        UiVelloText {
                            value: "Scene in bevy_ui".to_string(),
                            text_align: VelloTextAlign::Middle,
                            style: VelloTextStyle {
                                font_size: 14.,
                                ..default()
                            },
                            ..default()
                        },
                        VelloTextAnchor::Center,
                    ));
                });

            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            ..default()
                        },
                        UiVelloSvg(
                            asset_server.load("embedded://scaling/assets/Ghostscript_Tiger.svg"),
                        ),
                        BorderColor::all(css::FUCHSIA.with_alpha(0.5)),
                    ));

                    parent.spawn((
                        ContentSize::default(),
                        Node::default(),
                        UiVelloText {
                            value: "SVG in bevy_ui".to_string(),
                            text_align: VelloTextAlign::Middle,
                            style: VelloTextStyle {
                                font_size: 14.,
                                ..default()
                            },
                            ..default()
                        },
                        VelloTextAnchor::Center,
                    ));
                });

            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            ..default()
                        },
                        UiVelloLottie(asset_server.load("embedded://scaling/assets/Tiger.json")),
                    ));

                    parent.spawn((
                        ContentSize::default(),
                        Node::default(),
                        UiVelloText {
                            value: "Lottie in bevy_ui".to_string(),
                            text_align: VelloTextAlign::Middle,
                            style: VelloTextStyle {
                                font_size: 14.,
                                ..default()
                            },
                            ..default()
                        },
                        VelloTextAnchor::Center,
                    ));
                });
        });
}

fn spawn_scenes(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn((
            VelloScene2d::new(),
            Aabb::from_min_max(Vec3::new(-50.0, -50.0, 0.0), Vec3::new(50.0, 50.0, 0.0)),
            Transform::from_xyz(-CELL_WIDTH, -CELL_HEIGHT, 0.0),
            RotateThing {
                initial_scale: Vec3::splat(1.0),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloText2d {
                    value: "Scene in world space".to_string(),
                    text_align: VelloTextAlign::Middle,
                    style: VelloTextStyle {
                        font_size: 14.,
                        ..default()
                    },
                    ..default()
                },
                VelloTextAnchor::Center,
            ));
        });

    commands
        .spawn((
            VelloSvg2d(asset_server.load("embedded://scaling/assets/Ghostscript_Tiger.svg")),
            Transform::from_xyz(0.0, -CELL_HEIGHT, 0.0).with_scale(Vec3::splat(0.1)),
            RotateThing {
                initial_scale: Vec3::splat(0.1),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloText2d {
                    value: "SVG in world space".to_string(),
                    text_align: VelloTextAlign::Middle,
                    style: VelloTextStyle {
                        font_size: 14.,
                        ..default()
                    },
                    ..default()
                },
                VelloTextAnchor::Center,
                Transform::from_scale(Vec3::splat(10.)),
            ));
        });

    commands
        .spawn((
            VelloLottie2d(asset_server.load("embedded://scaling/assets/Tiger.json")),
            Transform::from_xyz(CELL_WIDTH, -CELL_HEIGHT, 0.0).with_scale(Vec3::splat(0.1)),
            RotateThing {
                initial_scale: Vec3::splat(0.1),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloText2d {
                    value: "Lottie in world space".to_string(),
                    text_align: VelloTextAlign::Middle,
                    style: VelloTextStyle {
                        font_size: 14.,
                        ..default()
                    },
                    ..default()
                },
                VelloTextAnchor::Center,
                Transform::from_scale(Vec3::splat(10.)),
            ));
        });
}

fn simple_non_ui_animation(mut scene_q: Query<&mut VelloScene2d, Without<Node>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    for mut scene in scene_q.iter_mut() {
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
    }
}

fn simple_ui_animation(mut scene_q: Query<&mut UiVelloScene>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    for mut scene in scene_q.iter_mut() {
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
            &kurbo::RoundedRect::new(0., 0., 100.0, 100.0, (sin_time as f64) * 50.0),
        );
    }
}

#[derive(Component, Clone)]
pub struct RotateThing {
    initial_scale: Vec3,
}

fn rotate(mut rotate_q: Query<&mut Transform, With<RotateThing>>, time: Res<Time>) {
    for mut transform in rotate_q.iter_mut() {
        transform.rotate_z(-0.5 * time.delta_secs());
    }
}

fn scale_control(
    mut scalable_query: Query<(&mut Transform, &RotateThing)>,
    mut keyboard_event_reader: MessageReader<KeyboardInput>,
) {
    for event in keyboard_event_reader.read() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::Digit1 => {
                    // Scale down
                    for (mut transform, _) in scalable_query.iter_mut() {
                        transform.scale *= 0.9;
                    }
                }
                KeyCode::Digit2 => {
                    // Scale up
                    for (mut transform, _) in scalable_query.iter_mut() {
                        transform.scale *= 1.1;
                    }
                }
                KeyCode::Digit3 => {
                    // Reset to original scales for each item type
                    for (mut transform, thing) in scalable_query.iter_mut() {
                        transform.scale = thing.initial_scale;
                    }
                }
                _ => {}
            }
        }
    }
}
