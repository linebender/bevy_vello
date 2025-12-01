use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    color::palettes::css,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    ui::ContentSize,
    window::WindowResolution,
};
use bevy_vello::{
    VelloPlugin,
    prelude::*,
    render::{SkipScaling, VelloScreenScale, VelloWorldScale},
};

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
            spawn_bevy_ui,
            spawn_screen_space,
            spawn_scenes,
            spawn_instructions,
        ),
    )
    .insert_resource(VelloScreenScale(1.))
    .insert_resource(VelloWorldScale(1.))
    .add_systems(
        Update,
        (
            rotate,
            simple_ui_animation,
            simple_non_ui_animation,
            scale_control,
        ),
    );
    embedded_asset!(app, "assets/svg/fountain.svg");
    embedded_asset!(app, "assets/lottie/Tiger.json");
    app.run();
}

const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEIGHT: f32 = 720.0;
const CELL_WIDTH: f32 = SCREEN_WIDTH / 4.0;
const CELL_HEIGHT: f32 = SCREEN_HEIGHT / 4.0;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
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
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor::all(css::FUCHSIA),
        SkipScaling,
        VelloTextSection {
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
            height: Val::Percent(33.0),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    RotateThing,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        VelloScene::new(),
                        BorderColor::all(css::FUCHSIA.with_alpha(0.5)),
                    ));

                    parent.spawn((
                        ContentSize::default(),
                        Node {
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(css::FUCHSIA.with_alpha(0.5)),
                        VelloTextSection {
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
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    RotateThing,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        VelloSvgHandle(
                            asset_server.load("embedded://scaling/assets/svg/fountain.svg"),
                        ),
                        BorderColor::all(css::FUCHSIA.with_alpha(0.5)),
                    ));

                    parent.spawn((
                        ContentSize::default(),
                        Node {
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(css::BLACK.with_alpha(0.5)),
                        VelloTextSection {
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
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    RotateThing,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        VelloLottieHandle(
                            asset_server.load("embedded://scaling/assets/lottie/Tiger.json"),
                        ),
                        BorderColor::all(css::FUCHSIA.with_alpha(0.5)),
                    ));

                    parent.spawn((
                        ContentSize::default(),
                        Node {
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(css::BLACK.with_alpha(0.5)),
                        VelloTextSection {
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

fn spawn_screen_space(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn((
            VelloRenderSpace::Screen,
            VelloScene::new(),
            Transform::from_xyz(CELL_WIDTH, SCREEN_HEIGHT / 2., 0.0),
            RotateThing,
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloRenderSpace::Screen,
                VelloTextSection {
                    value: "Scene in screen space".to_string(),
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
            VelloRenderSpace::Screen,
            VelloSvgHandle(asset_server.load("embedded://scaling/assets/svg/fountain.svg")),
            Transform::from_xyz(CELL_WIDTH * 2., SCREEN_HEIGHT / 2., 0.0),
            RotateThing,
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloRenderSpace::Screen,
                VelloTextSection {
                    value: "SVG in screen space".to_string(),
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
            VelloRenderSpace::Screen,
            VelloLottieHandle(asset_server.load("embedded://scaling/assets/lottie/Tiger.json")),
            Transform::from_xyz(CELL_WIDTH * 3., SCREEN_HEIGHT / 2., 0.0)
                .with_scale(Vec3::splat(0.1)),
            RotateThing,
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloRenderSpace::Screen,
                VelloTextSection {
                    value: "Lottie in screen space".to_string(),
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

fn spawn_scenes(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn((
            VelloScene::new(),
            RotateThing,
            Transform::from_xyz(-CELL_WIDTH, -CELL_HEIGHT, 0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloTextSection {
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
            VelloSvgHandle(asset_server.load("embedded://scaling/assets/svg/fountain.svg")),
            Transform::from_xyz(0.0, -CELL_HEIGHT, 0.0),
            RotateThing,
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloTextSection {
                    value: "SVG in world space".to_string(),
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
            VelloLottieHandle(asset_server.load("embedded://scaling/assets/lottie/Tiger.json")),
            RotateThing,
            Transform::from_xyz(CELL_WIDTH, -CELL_HEIGHT, 0.0).with_scale(Vec3::splat(0.1)),
        ))
        .with_children(|parent| {
            parent.spawn((
                VelloTextSection {
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

fn simple_non_ui_animation(mut scene_q: Query<&mut VelloScene, Without<Node>>, time: Res<Time>) {
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
            &kurbo::RoundedRect::new(-25.0, -25.0, 25.0, 25.0, (sin_time as f64) * 25.0),
        );
    }
}

fn simple_ui_animation(mut scene_q: Query<&mut VelloScene>, time: Res<Time>) {
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
            &kurbo::RoundedRect::new(0., 0., 100.0, 100.0, (sin_time as f64) * 25.0),
        );
    }
}

#[derive(Component, Clone)]
pub struct RotateThing;

fn rotate(mut rotate_q: Query<&mut Transform, With<RotateThing>>, time: Res<Time>) {
    for mut transform in rotate_q.iter_mut() {
        transform.rotate_z(-0.5 * time.delta_secs());
    }
}

fn scale_control(
    mut commands: Commands,
    world_scale: Res<VelloWorldScale>,
    screen_scale: Res<VelloScreenScale>,
    mut keyboard_event_reader: MessageReader<KeyboardInput>,
) {
    for event in keyboard_event_reader.read() {
        if event.state == ButtonState::Pressed {
            if event.key_code == KeyCode::Digit1 && world_scale.0 > 0.1 {
                commands.insert_resource(VelloWorldScale(world_scale.0 - 0.1));
                commands.insert_resource(VelloScreenScale(screen_scale.0 - 0.1));
            }

            if event.key_code == KeyCode::Digit2 && world_scale.0 < 10.0 {
                commands.insert_resource(VelloWorldScale(world_scale.0 + 0.1));
                commands.insert_resource(VelloScreenScale(screen_scale.0 + 0.1));
            }

            if event.key_code == KeyCode::Digit3 {
                commands.insert_resource(VelloWorldScale(1.0));
                commands.insert_resource(VelloScreenScale(1.0));
            }
        }
    }
}
