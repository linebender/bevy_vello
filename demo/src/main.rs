use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_vello::{
    debug::DebugVisualizations, ColorPaletteSwap, Origin, PlaybackDirection, PlaybackSettings,
    State, StateMachine, StateTransition, VelloAsset, VelloAssetBundle, VelloPlugin, VelloText,
    VelloTextBundle,
};

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(
            Update,
            (
                camera_system,
                drag_and_drop,
                print_metadata,
                dynamic_color_remapping,
            ),
        )
        .run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(VelloAssetBundle {
            origin: bevy_vello::Origin::Center,
            // Can only load *.json (Lottie animations) and *.svg (static vector graphics)
            vector: asset_server.load("../assets/squid.json"),
            debug_visualizations: DebugVisualizations::Visible,
            ..default()
        })
        .insert(
            StateMachine::new("demo-sm", "fast")
                .with_state(
                    State::new("slow", asset_server.load("../assets/squid.json"))
                        .with_transition(StateTransition::OnComplete { state: "fast" })
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Normal,
                            speed: 0.2,
                            looping: true,
                            segments: 0.0..96.0,
                        }),
                )
                .with_state(
                    State::new("fast", asset_server.load("../assets/squid.json"))
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Reverse,
                            speed: 1.0,
                            looping: true,
                            segments: 0.0..96.0,
                        })
                        .with_transition(StateTransition::OnComplete { state: "slow" }),
                ),
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

fn dynamic_color_remapping(
    mut commands: Commands,
    mut q: Query<Entity, With<Handle<VelloAsset>>>,
    time: Res<Time>,
) {
    for e in q.iter_mut() {
        commands.entity(e).insert({
            const CYCLE_TIME_SECS: f32 = 10.0;
            let color = Color::hsl(
                time.elapsed_seconds() / CYCLE_TIME_SECS * 360.0 % 360.0,
                1.0,
                0.5,
            );
            ColorPaletteSwap::empty()
                .add("suckers ", color)
                .add("suckers Flip", color)
        });
    }
}

fn print_metadata(
    mut asset_ev: EventReader<AssetEvent<VelloAsset>>,
    assets: Res<Assets<VelloAsset>>,
) {
    for ev in asset_ev.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let asset = assets.get(*id).unwrap();
            if let Some(metadata) = asset.metadata() {
                info!(
                    "Animated asset loaded. Layers:\n{:#?}",
                    metadata.get_layers()
                );
            }
        }
    }
}

/// Transform the camera to the center of the vector graphic apply zooming
fn camera_system(
    mut query: Query<(&GlobalTransform, &mut Handle<VelloAsset>, &Origin)>,
    mut query_cam: Query<&mut Transform, (With<Camera>, Without<Handle<VelloAsset>>)>,
    vectors: ResMut<Assets<VelloAsset>>,
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
    //projection.scale = 2.0 * time.elapsed_seconds().sin().clamp(0.2, 0.8);

    // Set the camera position to the center point of the vector
    if let Some(vector) = vectors.get(vector.as_ref()) {
        camera_transform.translation = vector
            .center_in_world(target_transform, origin)
            .extend(camera_transform.translation.z);
    }
}

/// Drag and drop any SVG or Lottie JSON asset into the window and change the displayed asset
fn drag_and_drop(
    mut query: Query<&mut Handle<VelloAsset>>,
    asset_server: ResMut<AssetServer>,
    mut dnd_evr: EventReader<FileDragAndDrop>,
) {
    let Ok(mut vector) = query.get_single_mut() else {
        return;
    };
    for ev in dnd_evr.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = ev else {
            continue;
        };
        let new_handle = asset_server.load(path_buf.clone());
        *vector = new_handle;
    }
}
