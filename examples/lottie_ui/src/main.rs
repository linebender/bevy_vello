use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    color::palettes::css,
    prelude::*,
};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, enable_debug)
    .add_systems(Startup, setup_camera)
    .add_systems(Startup, load_lottie);
    embedded_asset!(app, "assets/Tiger.json");
    app.run();
}

fn enable_debug(mut options: ResMut<UiDebugOptions>) {
    options.enabled = true;
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_lottie(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let lottie_handle = asset_server.load("embedded://lottie_ui/assets/Tiger.json");

    // Create a 3x3 grid to demonstrate Lottie rendering in different positions
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::percent(3, 33.33),
            grid_template_rows: RepeatedGridTrack::percent(3, 33.33),
            ..default()
        })
        .with_children(|parent| {
            // Create 9 cells with Lottie animations
            for _ in 0..9 {
                parent.spawn((
                    Node {
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BorderColor::all(css::FUCHSIA.with_alpha(0.5)),
                    BackgroundColor(css::DARK_SLATE_GRAY.with_alpha(0.3).into()),
                    UiVelloLottie(lottie_handle.clone()),
                ));
            }
        });
}
