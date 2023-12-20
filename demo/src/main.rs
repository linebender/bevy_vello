use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_vello::{
    debug::DebugVisualizations, ColorPaletteSwap, Origin, VelloPlugin, VelloText, VelloTextBundle,
    VelloVector, VelloVectorBundle,
};

const BODY_BASE: Color = Color::rgba(129. / 255., 94. / 255., 1.0, 1.0);
const BODY_DARK: Color = Color::rgba(73. / 255., 20. / 255., 165. / 255., 1.0);
const TENTACLES_HIGHLIGHT: Color = Color::rgba(178. / 255., 168. / 255., 1.0, 1.0);
const SUCKERS: Color = Color::rgba(235. / 255., 189. / 255., 1.0, 1.0);

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(AssetPlugin { ..default() }))
        .add_plugins(VelloPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(Update, (camera_system, drag_and_drop))
        .run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(VelloVectorBundle {
            origin: bevy_vello::Origin::Center,
            // Can only load *.json (Lottie animations) and *.svg (static vector graphics)
            vector: asset_server.load("../assets/squid.json"),
            debug_visualizations: DebugVisualizations::Visible,
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
            content: "hello vello".to_string(),
            size: 100.0,
        },
        ..default()
    });
}

/// Transform the camera to the center of the vector graphic apply zooming
fn camera_system(
    mut query: Query<(&GlobalTransform, &mut Handle<VelloVector>, &Origin)>,
    mut query_cam: Query<&mut Transform, (With<Camera>, Without<Handle<VelloVector>>)>,
    vectors: ResMut<Assets<VelloVector>>,
    mut q: Query<&mut OrthographicProjection, With<Camera>>,
    time: Res<Time>,
) {
    let Ok(mut projection) = q.get_single_mut() else {
        return;
    };
    let Ok(mut camera_transform) = query_cam.get_single_mut() else {
        return;
    };
    let Ok((target_transform, vector, origin)) = query.get_single_mut() else {
        return;
    };

    // Zoom in & out to demonstrate scalability and show the vector graphic's viewbox/anchor point
    projection.scale = 2.0 * time.elapsed_seconds().sin().clamp(0.2, 0.8);

    // Set the camera position to the center point of the vector
    if let Some(vector) = vectors.get(vector.as_ref()) {
        camera_transform.translation = vector
            .center_in_world(target_transform, origin)
            .extend(camera_transform.translation.z);
    }
}

/// Drag and drop any SVG or Lottie JSON asset into the window and change the displayed asset
fn drag_and_drop(
    mut query: Query<(&Transform, &mut Handle<VelloVector>)>,
    _asset_server: ResMut<AssetServer>,
    mut _dnd_evr: EventReader<FileDragAndDrop>,
) {
    let Ok((_, mut _vector)) = query.get_single_mut() else {
        return;
    };

    // todo: this broke after migration to bevy 0.12
    // for ev in dnd_evr.iter() {
    //     if let FileDragAndDrop::DroppedFile { path_buf, .. } = ev {
    //         let new_handle = asset_server.load(path_buf.to_str().unwrap());
    //         *vector = new_handle;
    //     }
    // }
}
