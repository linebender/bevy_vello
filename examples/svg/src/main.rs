use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
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
    .add_systems(Startup, load_svg)
    .add_systems(Update, (rotate, gizmos));
    embedded_asset!(app, "assets/fountain.svg");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_svg(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    // You can also use `VelloSvgBundle`
    commands
        .spawn((
            VelloSvgHandle(asset_server.load("embedded://svg/assets/fountain.svg")),
            VelloSvgAnchor::Center,
        ))
        .insert(Transform::from_scale(Vec3::splat(1.0)));
}

fn rotate(mut svg: Single<&mut Transform, With<VelloSvgHandle>>, time: Res<Time>) {
    svg.rotate_z(-0.5 * time.delta_secs());
}

fn gizmos(
    svg: Single<(&VelloSvgHandle, &GlobalTransform)>,
    assets: Res<Assets<VelloSvg>>,
    mut gizmos: Gizmos,
) {
    let (svg, gtransform) = *svg;
    let Some(svg) = assets.get(svg.id()) else {
        return;
    };

    gizmos.rect_2d(
        Isometry2d::new(
            gtransform.translation().xy(),
            Rot2::radians(gtransform.rotation().to_scaled_axis().z),
        ),
        Vec2::new(svg.width, svg.height) * gtransform.scale().xy(),
        Color::WHITE,
    );
}
