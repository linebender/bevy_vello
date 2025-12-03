use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    camera::primitives::Aabb,
    prelude::*,
    window::WindowResolution,
};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(512, 512),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, setup_camera)
    .add_systems(Startup, load_lottie)
    .add_systems(Update, gizmos);
    embedded_asset!(app, "assets/Tiger.json");
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_lottie(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn(VelloLottie2d(
            asset_server.load("embedded://lottie/assets/Tiger.json"),
        ))
        .insert(Transform::from_scale(Vec3::splat(0.5)));
}

fn gizmos(
    lottie_entities: Query<(&Aabb, &GlobalTransform), With<VelloLottie2d>>,
    mut gizmos: Gizmos,
) {
    for (aabb, transform) in lottie_entities.iter() {
        gizmos.rect_2d(
            Isometry2d::new(
                transform.translation().xy(),
                Rot2::radians(transform.rotation().to_scaled_axis().z),
            ),
            aabb.half_extents.to_vec3().xy(),
            Color::WHITE,
        );
    }
}
