use bevy::prelude::*;
use bevy_vello::{BevyVelloPlugin, VelloText, VelloTextBundle, VelloVector, VelloVectorBundle};

fn setup_vello(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(VelloVectorBundle {
        layer: bevy_vello::Layer::Background,
        svg: asset_server.load("../assets/squid.json"),
        debug_visualizations: bevy_vello::DebugVisualizations::Visible,
        ..default()
    });

    commands.spawn(VelloTextBundle {
        font: asset_server.load("../assets/Rubik-Medium.vttf"),
        text: VelloText {
            content: "squid".to_string(),
            size: 320.0,
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: asset_server.load("branding/icon.png"),
        ..default()
    });
}

fn camera_to_asset_center(
    query: Query<(&Transform, &Handle<VelloVector>)>,
    mut query_cam: Query<&mut Transform, (With<Camera>, Without<Handle<VelloVector>>)>,
    vectors: Res<Assets<VelloVector>>,
    mut q: Query<&mut OrthographicProjection, With<Camera>>,
) {
    let mut projection = q.single_mut();

    // example: zoom out
    projection.scale = 2.0;

    let mut camera_transform = query_cam.single_mut();
    let (&(mut target_transform), vector) = query.single();
    if let Some(vector) = vectors.get(vector) {
        target_transform.translation.y += vector.height * target_transform.scale.y / 2.0;
        camera_transform.translation.x = target_transform.translation.x;
        camera_transform.translation.y = target_transform.translation.y;
        // NOTE: you should not set the camera z or the vello canvas will get clipped out of view
    }
}

////////////////////////////////////////////////////
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // This tells the AssetServer to watch for changes to assets.
            // It enables our scenes to automatically reload in game when we modify their files.
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(BevyVelloPlugin)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Update, camera_to_asset_center)
        .add_systems(Startup, setup_vello)
        .run();
}
