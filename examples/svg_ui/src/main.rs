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
    .add_systems(Startup, setup_camera)
    .add_systems(Startup, enable_debug)
    .add_systems(Startup, load_svg);
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn enable_debug(mut options: ResMut<UiDebugOptions>) {
    options.enabled = true;
}

fn load_svg(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let svg_handle = asset_server.load("embedded://svg_ui/assets/Ghostscript_Tiger.svg");

    // Create a 3x3 grid to demonstrate SVG rendering in different positions
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
            // Create 9 cells with SVGs
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
                    UiVelloSvg(svg_handle.clone()),
                ));
            }
        });
}
