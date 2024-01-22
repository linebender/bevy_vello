use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_vello::{
    debug::DebugVisualizations, AnimationController, AnimationState, AnimationTransition,
    ColorPaletteSwap, PlaybackDirection, PlaybackSettings, VelloAsset, VelloAssetBundle,
    VelloPlugin, VelloText, VelloTextBundle,
};

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_plugins(bevy_pancam::PanCamPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(
            Update,
            (drag_and_drop, print_metadata, dynamic_color_remapping),
        )
        .run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), bevy_pancam::PanCam::default()));
    commands
        .spawn(VelloAssetBundle {
            origin: bevy_vello::Origin::Center,
            vector: asset_server.load("../assets/squid.json"),
            debug_visualizations: DebugVisualizations::Visible,
            ..default()
        })
        .insert(
            AnimationController::new("slow")
                .with_state(
                    AnimationState::new("slow")
                        .with_asset(asset_server.load("../assets/squid.json"))
                        .with_transition(AnimationTransition::OnMouseEnter { state: "fast" })
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Normal,
                            speed: 0.2,
                            looping: true,
                            segments: 0.0..96.0,
                        }),
                )
                .with_state(
                    AnimationState::new("fast")
                        .with_asset(asset_server.load("../assets/squid.json"))
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Reverse,
                            speed: 1.0,
                            looping: true,
                            segments: 0.0..96.0,
                        })
                        .with_transition(AnimationTransition::OnMouseLeave { state: "slow" }),
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

    commands
        .spawn(VelloAssetBundle {
            origin: bevy_vello::Origin::Center,
            vector: asset_server.load("../assets/Front.json"),
            transform: Transform::from_translation(Vec3::new(300.0, 200.0, 0.0)),
            debug_visualizations: DebugVisualizations::Visible,
            ..default()
        })
        .insert(
            AnimationController::new("slow")
                .with_state(
                    AnimationState::new("slow")
                        .with_asset(asset_server.load("../assets/Front.json"))
                        .with_transition(AnimationTransition::OnMouseEnter { state: "fast" })
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Normal,
                            speed: 0.2,
                            looping: true,
                            segments: 0.0..96.0,
                        }),
                )
                .with_state(
                    AnimationState::new("fast")
                        .with_asset(asset_server.load("../assets/Front.json"))
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Reverse,
                            speed: 1.0,
                            looping: true,
                            segments: 0.0..96.0,
                        })
                        .with_transition(AnimationTransition::OnMouseLeave { state: "slow" }),
                ),
        );

    commands
        .spawn(VelloAssetBundle {
            origin: bevy_vello::Origin::Center,
            vector: asset_server.load("../assets/example.json"),
            transform: Transform::from_translation(Vec3::new(-750.0, 0.0, 0.0))
                .with_scale(Vec3::splat(30.0)),
            debug_visualizations: DebugVisualizations::Visible,
            ..default()
        })
        .insert(
            AnimationController::new("stopped")
                .with_state(
                    AnimationState::new("stopped")
                        .with_asset(asset_server.load("../assets/example.json"))
                        .with_transition(AnimationTransition::OnMouseEnter { state: "play" })
                        .with_playback_settings(PlaybackSettings {
                            autoplay: false,
                            ..default()
                        }),
                )
                .with_state(
                    AnimationState::new("play")
                        .with_asset(asset_server.load("../assets/example.json"))
                        .with_transition(AnimationTransition::OnMouseLeave { state: "rev" })
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Normal,
                            looping: false,
                            ..default()
                        }),
                )
                .with_state(
                    AnimationState::new("rev")
                        .with_asset(asset_server.load("../assets/example.json"))
                        .with_playback_settings(PlaybackSettings {
                            autoplay: true,
                            direction: PlaybackDirection::Reverse,
                            looping: false,
                            ..default()
                        })
                        .with_transition(AnimationTransition::OnComplete { state: "stopped" }),
                ),
        );
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
                    metadata.get_layers().collect::<Vec<_>>()
                );
            }
        }
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
