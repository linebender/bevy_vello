use bevy::prelude::*;
use bevy_vello::{
    BevyVelloPlugin, ColorPaletteSwap, VelloText, VelloTextBundle, VelloVector, VelloVectorBundle,
};

const BODY_BASE: Color = Color::rgba(129. / 255., 94. / 255., 255. / 255., 255. / 255.);
const BODY_DARK: Color = Color::rgba(73. / 255., 20. / 255., 165. / 255., 255. / 255.);
const TENTACLES_HIGHLIGHT: Color = Color::rgba(178. / 255., 168. / 255., 255. / 255., 255. / 255.);
const SUCKERS: Color = Color::rgba(235. / 255., 189. / 255., 255. / 255., 255. / 255.);

fn setup_vello(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(VelloVectorBundle {
            layer: bevy_vello::Layer::Background,
            svg: asset_server.load("../assets/squid.json"),
            debug_visualizations: bevy_vello::DebugVisualizations::Visible,
            ..default()
        })
        .insert(
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
