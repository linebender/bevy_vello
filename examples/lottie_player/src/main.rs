mod ui;
use bevy::{
    asset::{AssetMetaCheck, embedded_asset, io::embedded::EmbeddedAssetRegistry},
    color::palettes::css,
    diagnostic::DiagnosticsStore,
    prelude::*,
};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(VelloPlugin::default())
    .init_resource::<EmbeddedAssetRegistry>()
    .add_systems(Startup, setup_camera)
    .add_systems(Startup, enable_debug)
    .add_systems(Startup, setup_vector_graphics)
    .add_systems(EguiPrimaryContextPass, ui::controls_ui);

    embedded_asset!(app, "assets/calendar.json");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn enable_debug(mut options: ResMut<UiDebugOptions>, mut config: ResMut<GizmoConfigStore>) {
    options.enabled = true;
    config.config_mut::<AabbGizmoConfigGroup>().1.draw_all = true;
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(400.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-200.0), // Half of width
                    top: Val::Px(-200.0),  // Half of height
                    ..default()
                },
                ..default()
            },
            UiVelloLottie(asset_server.load("embedded://lottie_player/assets/calendar.json")),
        ))
        .insert(
            LottiePlayer::<UiVelloLottie>::new("stopped")
                .with_state(
                    PlayerState::new("stopped")
                        .playback_options(PlaybackOptions {
                            autoplay: false,
                            ..default()
                        })
                        .theme(Theme::new().add("calendar", css::YELLOW.into()))
                        .transition(PlayerTransition::OnMouseEnter { state: "play" })
                        .reset_playhead_on_start(),
                )
                .with_state(
                    PlayerState::new("play")
                        .playback_options(PlaybackOptions {
                            looping: PlaybackLoopBehavior::DoNotLoop,
                            speed: 0.75,
                            ..default()
                        })
                        .theme(Theme::new().add("calendar", css::LIME.into()))
                        .transition(PlayerTransition::OnMouseLeave { state: "rev" }),
                )
                .with_state(
                    PlayerState::new("rev")
                        .playback_options(PlaybackOptions {
                            looping: PlaybackLoopBehavior::DoNotLoop,
                            direction: PlaybackDirection::Reverse,
                            speed: 0.75,
                            ..default()
                        })
                        .theme(Theme::new().add("calendar", css::RED.into()))
                        .transition(PlayerTransition::OnMouseEnter { state: "play" })
                        .transition(PlayerTransition::OnComplete { state: "stopped" }),
                ),
        );
}
