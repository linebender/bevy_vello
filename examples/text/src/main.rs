use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    prelude::*,
    ui::ContentSize,
};
use bevy_vello::{VelloPlugin, prelude::*, text::VelloTextAnchor};

const EMBEDDED_FONT: &str = "embedded://text/assets/RobotoFlex-VariableFont_GRAD,XOPQ,XTRA,YOPQ,YTAS,YTDE,YTFI,YTLC,YTUC,opsz,slnt,wdth,wght.ttf";

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
    .add_systems(Update, (toggle_animations, animate_axes, gizmos).chain())
    .init_resource::<AnimationToggles>();
    embedded_asset!(
        app,
        "assets/RobotoFlex-VariableFont_GRAD,XOPQ,XTRA,YOPQ,YTAS,YTDE,YTFI,YTLC,YTUC,opsz,slnt,wdth,wght.ttf"
    );
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn setup_worldspace_text(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(VelloTextBundle {
        text: VelloTextSection {
            value: "bevy_vello using Bevy's default font".to_string(),
            style: VelloTextStyle {
                font_size: 24.0,
                ..default()
            },
        },
        text_anchor: VelloTextAnchor::Center,
        transform: Transform::from_xyz(0.0, -100.0, 0.0),
        ..default()
    });

    commands.spawn(VelloTextBundle {
        text: VelloTextSection {
            value: "bevy_vello using RobotoFlex-VariableFont".to_string(),
            style: VelloTextStyle {
                font: asset_server.load(EMBEDDED_FONT),
                font_size: 48.0,
                ..default()
            },
        },
        text_anchor: VelloTextAnchor::Center,
        ..default()
    });
}

#[derive(Debug, Default, Resource)]
struct AnimationToggles {
    pub weight: bool,
    pub width: bool,
    pub slant: bool,
    pub grade: bool,
    pub thick_stroke: bool,
    pub thin_stroke: bool,
    pub counter_width: bool,
    pub uppercase_height: bool,
    pub lowercase_height: bool,
    pub ascender_height: bool,
    pub descender_depth: bool,
    pub figure_height: bool,
}

fn toggle_animations(
    mut toggles: ResMut<AnimationToggles>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.any_just_pressed([
        KeyCode::KeyQ,
        KeyCode::KeyW,
        KeyCode::KeyE,
        KeyCode::KeyR,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::KeyF,
        KeyCode::KeyZ,
        KeyCode::KeyX,
        KeyCode::KeyC,
        KeyCode::KeyV,
    ]) {
        for key in keyboard_input.get_just_pressed() {
            match key {
                KeyCode::KeyQ => toggles.weight = !toggles.weight,
                KeyCode::KeyW => toggles.width = !toggles.width,
                KeyCode::KeyE => toggles.slant = !toggles.slant,
                KeyCode::KeyR => toggles.grade = !toggles.grade,
                KeyCode::KeyA => toggles.thick_stroke = !toggles.thick_stroke,
                KeyCode::KeyS => toggles.thin_stroke = !toggles.thin_stroke,
                KeyCode::KeyD => toggles.counter_width = !toggles.counter_width,
                KeyCode::KeyF => toggles.uppercase_height = !toggles.uppercase_height,
                KeyCode::KeyZ => toggles.lowercase_height = !toggles.lowercase_height,
                KeyCode::KeyX => toggles.ascender_height = !toggles.ascender_height,
                KeyCode::KeyC => toggles.descender_depth = !toggles.descender_depth,
                KeyCode::KeyV => toggles.figure_height = !toggles.figure_height,
                _ => (),
            }
        }
    }
}

const ANIMATION_SPEED: f32 = 5.0;

fn animate_axes(
    time: Res<Time>,
    mut query: Query<&mut VelloTextSection>,
    animation_toggles: Res<AnimationToggles>,
) {
    let sin_time = (time.elapsed_secs() * ANIMATION_SPEED)
        .sin()
        .mul_add(0.5, 0.5);

    // https://fonts.google.com/specimen/Roboto+Flex/tester?query=variable
    let font_weight = sin_time.remap(0., 1., 100., 1000.);
    let font_width = sin_time.remap(0., 1., 25., 151.);
    let slant = sin_time.remap(0., 1., -10., 0.);
    let grade = sin_time.remap(0., 1., -200., 150.).round();
    let thick_stroke = sin_time.remap(0., 1., 27., 175.);
    let thin_stroke = sin_time.remap(0., 1., 25., 135.);
    let counter_width = sin_time.remap(0., 1., 323., 603.);
    let uppercase_height = sin_time.remap(0., 1., 528., 760.);
    let lowercase_height = sin_time.remap(0., 1., 416., 570.);
    let ascender_height = sin_time.remap(0., 1., 649., 854.);
    let descender_depth = sin_time.remap(0., 1., -98., -305.);
    let figure_height = sin_time.remap(0., 1., 560., 788.);

    for mut text_section in query.iter_mut() {
        if animation_toggles.weight {
            text_section.style.font_axes.weight = Some(font_weight);
        }

        if animation_toggles.width {
            text_section.style.font_axes.width = Some(font_width);
        }

        if animation_toggles.slant {
            text_section.style.font_axes.slant = Some(slant);
        }

        if animation_toggles.grade {
            text_section.style.font_axes.grade = Some(grade);
        }

        if animation_toggles.thick_stroke {
            text_section.style.font_axes.thick_stroke = Some(thick_stroke);
        }

        if animation_toggles.thin_stroke {
            text_section.style.font_axes.thin_stroke = Some(thin_stroke);
        }

        if animation_toggles.counter_width {
            text_section.style.font_axes.counter_width = Some(counter_width);
        }

        if animation_toggles.uppercase_height {
            text_section.style.font_axes.uppercase_height = Some(uppercase_height);
        }

        if animation_toggles.lowercase_height {
            text_section.style.font_axes.lowercase_height = Some(lowercase_height);
        }

        if animation_toggles.ascender_height {
            text_section.style.font_axes.ascender_height = Some(ascender_height);
        }

        if animation_toggles.descender_depth {
            text_section.style.font_axes.descender_depth = Some(descender_depth);
        }

        if animation_toggles.figure_height {
            text_section.style.font_axes.figure_height = Some(figure_height);
        }
    }
}

fn setup_screenspace_text(mut commands: Commands) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            row_gap: Val::Px(12.),
            column_gap: Val::Px(12.),
            padding: UiRect::new(Val::Px(12.), Val::Px(12.), Val::Px(12.), Val::Px(12.)),
            ..default()
        },))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::percent(4, 25.),
                    ..default()
                },))
                .with_children(|qwer| {
                    qwer.spawn(Node::default())
                        .with_child(Text::new("Q: weight"));
                    qwer.spawn(Node::default())
                        .with_child(Text::new("W: width"));
                    qwer.spawn(Node::default())
                        .with_child(Text::new("E: slant"));
                    qwer.spawn(Node::default())
                        .with_child(Text::new("R: grade"));
                });

            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::percent(4, 25.),
                        ..default()
                    },
                    ContentSize::default(),
                ))
                .with_children(|asdf| {
                    asdf.spawn(Node::default())
                        .with_child(Text::new("A: thick stroke"));
                    asdf.spawn(Node::default())
                        .with_child(Text::new("S: thin stroke"));
                    asdf.spawn(Node::default())
                        .with_child(Text::new("D: counter width"));
                    asdf.spawn(Node::default())
                        .with_child(Text::new("F: uppercase height"));
                });

            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::percent(4, 25.),
                        ..default()
                    },
                    ContentSize::default(),
                ))
                .with_children(|zxcv| {
                    zxcv.spawn(Node::default())
                        .with_child(Text::new("Z: lowercase height"));
                    zxcv.spawn(Node::default())
                        .with_child(Text::new("X: ascender height"));
                    zxcv.spawn(Node::default())
                        .with_child(Text::new("C: descender depth"));
                    zxcv.spawn(Node::default())
                        .with_child(Text::new("V: figure height"));
                });
        });
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
