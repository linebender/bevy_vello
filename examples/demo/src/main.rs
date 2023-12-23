use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_vello::{
    debug::DebugVisualizations, Origin, VelloPlugin, VelloText, VelloTextBundle, VelloVector,
    VelloVectorBundle,
};

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
    commands.spawn(VelloVectorBundle {
        origin: bevy_vello::Origin::Center,
        // Can only load *.json (Lottie animations) and *.svg (static vector graphics)
        vector: asset_server.load("../assets/squid.json"),
        debug_visualizations: DebugVisualizations::Visible,
        ..default()
    });

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
    asset_server: ResMut<AssetServer>,
    mut dnd_evr: EventReader<FileDragAndDrop>,
) {
    let Ok((_, mut vector)) = query.get_single_mut() else {
        return;
    };

    for ev in dnd_evr.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = ev {
            let new_handle = asset_server.load(path_buf.to_str().unwrap().to_owned());
            *vector = new_handle;
        }
    }
}
