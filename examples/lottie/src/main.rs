use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, load_lottie)
    .add_systems(Update, gizmos);
    embedded_asset!(app, "assets/Tiger.json");
    app.run();
}

fn load_lottie(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2d, VelloView));

    // You can also use `VelloLottieBundle`
    commands
        .spawn(VelloLottieHandle(
            asset_server.load("embedded://lottie/assets/Tiger.json"),
        ))
        .insert(Transform::from_scale(Vec3::splat(0.5)));
}

fn gizmos(
    svg: Single<(&VelloLottieHandle, &GlobalTransform)>,
    assets: Res<Assets<VelloLottie>>,
    mut gizmos: Gizmos,
) {
    let (lottie, gtransform) = *svg;
    let Some(lottie) = assets.get(svg.id()) else {
        return;
    };

    gizmos.rect_2d(
        Isometry2d::new(
            gtransform.translation().xy(),
            Rot2::radians(gtransform.rotation().to_scaled_axis().z),
        ),
        Vec2::new(lottie.width, lottie.height) * gtransform.scale().xy(),
        Color::WHITE,
    );
}
