use bevy::{diagnostic::DiagnosticsStore, prelude::*};
use bevy_vello::{VelloPlugin, prelude::*};

const SCENE_COUNT: usize = 5;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, simple_animation)
        .add_systems(Update, update_scene_count_ui);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
    for i in 0..SCENE_COUNT {
        commands.spawn((
            VelloScene2d::new(),
            Transform::from_translation(Vec3::new(i as f32 * 100.0 - 200.0, 0.0, 0.0)),
        ));
    }

    // UI Text displaying the scene count
    commands.spawn((
        Text::new("Total Scenes: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        DiagnosticsText,
    ));
}

#[derive(Component)]
struct DiagnosticsText;

fn simple_animation(mut query: Query<(&mut Transform, &mut VelloScene2d)>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);

    for (mut transform, mut scene) in query.iter_mut() {
        scene.reset();

        let c = Vec3::lerp(
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            sin_time + 0.5,
        );
        scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::default(),
            peniko::Color::new([c.x, c.y, c.z, 1.]),
            None,
            &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
        );

        transform.scale = Vec3::lerp(Vec3::splat(0.5), Vec3::ONE, sin_time);
        transform.translation.y = sin_time * 50.0;
        transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
    }
}

fn update_scene_count_ui(
    diagnostics: Res<DiagnosticsStore>,
    mut text: Single<&mut Text, With<DiagnosticsText>>,
) {
    let scene_count = diagnostics
        .get(&bevy_vello::render::diagnostics::WORLD_SCENE_COUNT)
        .and_then(|d| d.measurement())
        .map(|m| m.value)
        .unwrap_or(0.0);

    let path_segs_count = diagnostics
        .get(&bevy_vello::render::diagnostics::PATH_SEGMENTS_COUNT)
        .and_then(|d| d.measurement())
        .map(|m| m.value)
        .unwrap_or(0.0);

    text.0 = format!("Total scenes: {scene_count}\nTotal path segments: {path_segs_count}");
}
