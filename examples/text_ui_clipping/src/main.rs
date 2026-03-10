/// UI Text Clipping Demo
///
/// This example demonstrates how UI text clipping works in bevy_vello.
/// It shows how text outside a clipping rectangle is efficiently culled
/// using line metrics and precise bounds checking.
///
/// Controls:
/// - Arrow Keys: Move the text
/// - R: Reset position
/// - Space: Toggle auto-move animation
///
use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    color::palettes::css,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_vello::{VelloPlugin, prelude::*};

const DEJAVU_SANS: &str = "embedded://text_ui_clipping/assets/DejaVuSans.ttf";

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_plugins(FrameTimeDiagnosticsPlugin::default())
    .add_systems(Startup, setup)
    .add_systems(Update, (handle_input, update_position, update_info_text));
    embedded_asset!(app, "assets/DejaVuSans.ttf");
    app.run();
}

#[derive(Component)]
struct MovableText;

#[derive(Component)]
struct ClipContainer;

#[derive(Resource)]
struct TextPosition {
    offset: Vec2,
    auto_move: bool,
}

#[derive(Component)]
struct InfoText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, VelloView));

    // Instructions (UI overlay)
    commands.spawn((
        Text::new(
            "UI TEXT CLIPPING DEMO\n\
            Controls:\n\
            - Arrow Keys: Move text\n\
            - R: Reset position\n\
            - Space: Toggle auto-move\n\n\
            The RED box shows the clip rect.\n\
            Text outside is culled.",
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(css::WHITE.into()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
    ));

    // Info panel (bottom left)
    commands.spawn((
        Text::new("Offset: (0.0, 0.0)"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(css::YELLOW.into()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        InfoText,
    ));

    // Position state resource
    commands.insert_resource(TextPosition {
        offset: Vec2::ZERO,
        auto_move: false,
    });

    // Create a clipped container in the center
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                // Center it (accounting for size)
                left: Val::Px(600.0),
                top: Val::Px(300.0),
                // Fixed size for the clip rect
                width: Val::Px(400.0),
                height: Val::Px(300.0),
                // Red border to visualize the clip boundary
                border: UiRect::all(Val::Px(3.0)),
                // THIS IS THE KEY: overflow: clip() enables clipping!
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(css::RED),
            BackgroundColor(css::DARK_SLATE_GRAY.with_alpha(0.3).into()),
            ClipContainer,
        ))
        .with_children(|parent| {
            // Movable text inside the clipped container
            parent.spawn((
                UiVelloText {
                    value: create_demo_text(),
                    style: VelloTextStyle {
                        font: asset_server.load(DEJAVU_SANS),
                        font_size: 20.0,
                        ..default()
                    },
                    text_align: VelloTextAlign::Left,
                    max_advance: Some(800.0), // Wider than container to show wrapping
                },
                VelloTextAnchor::TopLeft,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                MovableText,
            ));
        });
}

fn create_demo_text() -> String {
    // Create text with multiple lines, some with extreme glyphs
    // Note: DejaVu Sans has limited support for Arabic, Devanagari, etc.
    // Some glyphs may render as boxes (tofu) but clipping still works!
    vec![
        "=== Line 1 ===",
        "Normal ASCII text",
        "",
        "=== Line 3 ===",
        "Stacked marks: A̎͂̃̄̅̆̇̈̉̊",
        "",
        "=== Line 5 ===",
        "Vietnamese: Ặ̀ Ệ́ Ơ̂́",
        "",
        "=== Line 7 ===",
        "Math: ∫∬∭∮∯ ∂∇∈∉∋∑∏√",
        "",
        "=== Line 9 ===",
        "Zalgo: H̷̢̧̛̰̹̺̻̼͇͈̊͋͌͏̀́̂̃ḛ̷̡̢̨̧̛̹̺̻̼͇͈͍͎͊͋͌͏̀́̂̃l̷̡̢̨̧̛̰̹̺̻̼͇͈͉͍͎͊͋͌͏̀́̂̃l̷̡̢̨̧̛̰̹̺̻̼͇͈͉͍͎͊͋͌͏̀́̂̃ơ̷̡̢̨̧̰̹̺̻̼͇͈͉͍͎͊͋͌͏̀́̂̃",
        "",
        "=== Line 11 ===",
        "Arabic: ڃ ڄ ڇ ڿ ۏ",
        "",
        "=== Line 13 ===",
        "Devanagari: ठ्ठ ड्ड ढ्ढ",
        "",
        "=== Line 15 ===",
        "Greek: Αα Ββ Γγ Δδ Εε Ζζ",
        "",
        "=== Line 17 ===",
        "Cyrillic: Аа Бб Вв Гг Дд",
        "",
        "=== Line 19 ===",
        "Accents: Àáâãäåāăą Ḉḱḽṁṅ",
        "",
        "=== Line 21 ===",
        "Symbols: ★☆♠♣♥♦←↑→↓",
        "",
        "=== Line 23 ===",
        "Box drawing: ┌─┐│└─┘",
        "",
        "=== Line 25 ===",
        "This text extends far",
        "beyond the visible area.",
        "Lines outside the red box",
        "are culled using line metrics!",
        "",
        "=== Line 30 ===",
        "Move with arrow keys to see",
        "how clipping works in real-time.",
        "Notice how entire lines are",
        "culled when they move outside",
        "the clip rectangle.",
        "",
        "=== Line 36 ===",
        "Even if some glyphs don't render",
        "(Arabic, Devanagari show as boxes),",
        "the line-level culling algorithm",
        "still works perfectly!",
        "",
        "=== End (Line 41) ===",
    ]
    .join("\n")
}

fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<TextPosition>) {
    let move_speed = 5.0;

    if keyboard.just_pressed(KeyCode::Space) {
        state.auto_move = !state.auto_move;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        state.offset = Vec2::ZERO;
        state.auto_move = false;
    }

    if keyboard.pressed(KeyCode::ArrowLeft) {
        state.offset.x -= move_speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        state.offset.x += move_speed;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        state.offset.y -= move_speed;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        state.offset.y += move_speed;
    }
}

fn update_position(
    time: Res<Time>,
    mut state: ResMut<TextPosition>,
    mut query: Query<&mut Node, With<MovableText>>,
) {
    if state.auto_move {
        // Circular motion to show clipping from all sides
        let t = time.elapsed_secs() * 0.5;
        state.offset.x = (t.cos() * 200.0).round();
        state.offset.y = (t.sin() * 150.0).round();
    }

    // Apply offset to text node
    if let Ok(mut node) = query.single_mut() {
        node.left = Val::Px(state.offset.x);
        node.top = Val::Px(state.offset.y);
    }
}

fn update_info_text(
    state: Res<TextPosition>,
    diagnostics: Res<DiagnosticsStore>,
    mut text_query: Query<&mut Text, With<InfoText>>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        let status = if state.auto_move { "ON" } else { "OFF" };

        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.smoothed())
            .unwrap_or(0.0);

        let frame_time = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|d| d.smoothed())
            .unwrap_or(0.0);

        let glyphs = diagnostics
            .get(&bevy_vello::render::diagnostics::GLYPH_COUNT)
            .and_then(|d| d.measurement())
            .map(|m| m.value)
            .unwrap_or(0.0);

        let glyph_runs = diagnostics
            .get(&bevy_vello::render::diagnostics::GLYPH_RUN_COUNT)
            .and_then(|d| d.measurement())
            .map(|m| m.value)
            .unwrap_or(0.0);

        text.0 = format!(
            "Offset: ({:.0}, {:.0})\n\
            Auto-move: {}\n\n\
            Performance:\n\
            - FPS: {:.1}\n\
            - Frame: {:.2}ms\n\
            - Glyphs: {:.0}\n\
            - Glyph runs: {:.0}\n\n\
            This demo shows UI text clipping.\n\
            Lines are culled using LineMetrics.\n\
            Move text to see culling in action!",
            state.offset.x, state.offset.y, status, fps, frame_time, glyphs, glyph_runs
        );
    }
}
