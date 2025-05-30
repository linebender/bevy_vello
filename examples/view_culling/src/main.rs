use std::ops::DerefMut;

use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    prelude::*,
    render::primitives::Aabb,
};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, load_view_culling)
    .add_systems(
        Update,
        (
            simple_animation,
            update_lottie_aab,
            log_vello_scene_view_visibility,
            update_svg_aab,
            update_text_aab,
        ),
    );

    embedded_asset!(app, "assets/lottie/Tiger.json");
    embedded_asset!(app, "assets/svg/fountain.svg");

    app.run();
}

fn log_vello_scene_view_visibility(
    query: Query<&ViewVisibility, (Changed<ViewVisibility>, With<VelloScene>)>,
) {
    for view_visibility in &query {
        if view_visibility.get() {
            println!("Scene is visible");
        } else {
            println!("Scene is not visible");
        }
    }
}

fn load_view_culling(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2d, VelloView));

    // TODO: add bb_in_screen_space and bb_in_world_space to VelloScene
    commands.spawn((
        VelloScene::new(),
        Aabb::from_min_max(Vec3::new(-50.0, -50.0, 0.0), Vec3::new(50.0, 50.0, 0.0)),
    ));

    // You can also use `VelloLottieBundle`
    commands
        .spawn(VelloLottieHandle(
            asset_server.load("embedded://view_culling/assets/lottie/Tiger.json"),
        ))
        .insert((Transform::from_scale(Vec3::splat(0.5)), Aabb::default()));

    commands
        .spawn(VelloSvgHandle(
            asset_server.load("embedded://view_culling/assets/svg/fountain.svg"),
        ))
        .insert((Transform::from_scale(Vec3::splat(5.0)), Aabb::default()));

    commands.spawn((
        VelloTextBundle {
            text: VelloTextSection {
                value: "View culled text".to_string(),
                style: VelloTextStyle {
                    font_size: 24.0,
                    ..default()
                },
                ..default()
            },
            text_anchor: VelloTextAnchor::Center,
            transform: Transform::default(),
            ..default()
        },
        Aabb::default(),
    ));
}

fn update_lottie_aab(
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    lottie_assets: Res<Assets<VelloLottie>>,
    mut lottie_q: Query<(&GlobalTransform, &VelloLottieHandle, &mut Aabb), Added<Aabb>>,
) {
    let (camera, camera_transform) = *camera;

    for (lottie_transform, handle, mut aabb) in &mut lottie_q {
        if let Some(vello_lottie) = lottie_assets.get(&handle.0) {
            if let Some(bb) =
                vello_lottie.bb_in_screen_space(lottie_transform, camera, camera_transform)
            {
                *aabb = Aabb::from_min_max(bb.min.extend(0.0), bb.max.extend(0.0));
            } else {
                *aabb = Aabb::default();
            }
        }
    }
}

fn update_svg_aab(
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    svg_assets: Res<Assets<VelloSvg>>,
    mut svg_q: Query<(&GlobalTransform, &VelloSvgHandle, &mut Aabb), Added<Aabb>>,
) {
    let (camera, camera_transform) = *camera;

    for (svg_transform, handle, mut aabb) in &mut svg_q {
        if let Some(vello_svg) = svg_assets.get(&handle.0) {
            if let Some(bb) = vello_svg.bb_in_screen_space(svg_transform, camera, camera_transform)
            {
                println!("SVG bounding box in screen space: {:?}", bb);
                *aabb = Aabb::from_min_max(bb.min.extend(0.0), bb.max.extend(0.0));
            } else {
                *aabb = Aabb::default();
            }
        }
    }
}

fn update_text_aab(
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    font_assets: Res<Assets<VelloFont>>,
    mut text_q: Query<(&GlobalTransform, &VelloTextSection, &mut Aabb), Added<Aabb>>,
) {
    let (camera, camera_transform) = *camera;

    for (text_transform, text_section, mut aabb) in &mut text_q {
        if let Some(font) = font_assets.get(&text_section.style.font) {
            if let Some(bb) =
                text_section.bb_in_screen_space(font, text_transform, camera, camera_transform)
            {
                *aabb = Aabb::from_min_max(bb.min.extend(0.0), bb.max.extend(0.0));
            } else {
                *aabb = Aabb::default();
            }
        }
    }
}

fn simple_animation(mut query_scene: Single<(&mut Transform, &mut VelloScene)>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let (transform, scene) = query_scene.deref_mut();
    // Reset scene every frame
    scene.reset();

    // Animate color green to blue
    let c = Vec3::lerp(
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        sin_time + 0.5,
    );

    // Animate the corner radius
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([c.x, c.y, c.z, 1.]),
        None,
        &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
    );

    transform.scale = Vec3::lerp(Vec3::ONE * 0.5, Vec3::ONE * 1.0, sin_time);
    transform.translation = Vec3::lerp(Vec3::X * -1000.0, Vec3::X * 1000.0, sin_time);
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}
