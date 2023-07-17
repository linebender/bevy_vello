use bevy::prelude::*;
use bevy_vello::{
    BevyVelloPlugin, ColorPaletteSwap, VelloText, VelloTextBundle, VelloVector, VelloVectorBundle,
};

const BODY_BASE: Color = Color::rgba(129. / 255., 94. / 255., 1.0, 1.0);
const BODY_DARK: Color = Color::rgba(73. / 255., 20. / 255., 165. / 255., 1.0);
const TENTACLES_HIGHLIGHT: Color = Color::rgba(178. / 255., 168. / 255., 1.0, 1.0);
const SUCKERS: Color = Color::rgba(235. / 255., 189. / 255., 1.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(BevyVelloPlugin)
        .add_startup_system(setup_vector_graphics)
        .add_system(camera_system)
        .add_system(drag_and_drop)
        .run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(VelloVectorBundle {
            layer: bevy_vello::Layer::Background,
            // Can only load *.json (Lottie animations) and *.svg (static vector graphics)
            svg: asset_server.load("../assets/squid.json"),
            debug_visualizations: bevy_vello::DebugVisualizations::Visible,
            ..default()
        })
        .insert(
            // This is optional, but demonstrates the ability to, at runtime, swap colors of individual layers on the vector animation programmatically
            ColorPaletteSwap::empty()
                .add("Arm", 1..=1, TENTACLES_HIGHLIGHT)
                .add("Arm", 0..=0, BODY_BASE)
                .add("Legs", 0..=0, BODY_BASE)
                .add("Legs", 1..=1, TENTACLES_HIGHLIGHT)
                .add("head", 4..=4, BODY_BASE)
                .add("head", 1..=3, BODY_DARK)
                .add("head", 5..=5, BODY_DARK)
                .add("suckers", 0..=16, SUCKERS),
        );

    commands.spawn(VelloTextBundle {
        font: asset_server.load("../assets/Rubik-Medium.vttf"),
        text: VelloText {
            content: "squid".to_string(),
            size: 320.0,
        },
        ..default()
    });
}

/// Transform the camera to the center of the vector graphic apply zooming
fn camera_system(
    mut query: Query<(&Transform, &mut Handle<VelloVector>)>,
    mut query_cam: Query<&mut Transform, (With<Camera>, Without<Handle<VelloVector>>)>,
    vectors: ResMut<Assets<VelloVector>>,
    mut q: Query<&mut OrthographicProjection, With<Camera>>,
    time: Res<Time>,
) {
    let mut projection = q.single_mut();

    // Zoom in & out to demonstrate scalability and show the vector graphic's viewbox/anchor point
    projection.scale = 2.0 * time.elapsed_seconds().cos();

    let mut camera_transform = query_cam.single_mut();
    let (&(mut target_transform), vector) = query.single_mut();
    if let Some(vector) = vectors.get(&vector) {
        target_transform.translation.y += vector.height * target_transform.scale.y / 2.0;
        camera_transform.translation = target_transform.translation;
    }
}

/// Drag and drop any SVG or Lottie JSON asset into the window and change the displayed asset
fn drag_and_drop(
    mut query: Query<(&Transform, &mut Handle<VelloVector>)>,
    asset_server: ResMut<AssetServer>,
    mut dnd_evr: EventReader<FileDragAndDrop>,
) {
    let (_, mut vector) = query.single_mut();

    for ev in dnd_evr.iter() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = ev {
            let new_handle = asset_server.load(path_buf.to_str().unwrap());
            *vector = new_handle;
        }
    }
}
