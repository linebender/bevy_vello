use bevy::{color::palettes::css, prelude::*};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, enable_debug)
        .add_systems(Startup, setup_ui)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn enable_debug(mut options: ResMut<UiDebugOptions>) {
    options.enabled = true;
}

fn setup_ui(mut commands: Commands) {
    // Create a 3x3 grid to demonstrate all 9 text anchors
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::percent(3, 33.33),
            grid_template_rows: RepeatedGridTrack::percent(3, 33.33),
            ..default()
        })
        .with_children(|parent| {
            // Row 1: Top anchors
            spawn_text_box(parent, VelloTextAnchor::TopLeft, "TopLeft");
            spawn_text_box(parent, VelloTextAnchor::Top, "Top");
            spawn_text_box(parent, VelloTextAnchor::TopRight, "TopRight");

            // Row 2: Middle anchors
            spawn_text_box(parent, VelloTextAnchor::Left, "Left");
            spawn_text_box(parent, VelloTextAnchor::Center, "Center");
            spawn_text_box(parent, VelloTextAnchor::Right, "Right");

            // Row 3: Bottom anchors
            spawn_text_box(parent, VelloTextAnchor::BottomLeft, "BottomLeft");
            spawn_text_box(parent, VelloTextAnchor::Bottom, "Bottom");
            spawn_text_box(parent, VelloTextAnchor::BottomRight, "BottomRight");
        });
}

fn spawn_text_box(parent: &mut ChildSpawnerCommands, anchor: VelloTextAnchor, label: &str) {
    parent.spawn((
        Node {
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(10.0)),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BorderColor::all(css::FUCHSIA.with_alpha(0.5)),
        BackgroundColor(css::DARK_SLATE_GRAY.with_alpha(0.3).into()),
        UiVelloText {
            value: label.to_string(),
            style: VelloTextStyle {
                font_size: 24.0,
                ..default()
            },
            ..default()
        },
        anchor,
    ));
}
