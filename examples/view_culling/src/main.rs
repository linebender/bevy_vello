use std::ops::DerefMut;

use bevy::{
    asset::{AssetMetaCheck, embedded_asset},
    camera::primitives::Aabb,
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
    .add_systems(Startup, load_view_culling)
    .add_systems(
        Update,
        (
            left_right,
            up_down,
            right_left,
            down_up,
            simple_animation,
            update_lottie_aabb,
            update_svg_aabb,
            update_text_aabb,
            draw_aabb_gizmos,
            log_visibility,
        )
            .chain(),
    );

    embedded_asset!(app, "assets/lottie/Tiger.json");
    embedded_asset!(app, "assets/svg/fountain.svg");

    app.run();
}

fn log_visibility(
    scene: Single<&ViewVisibility, With<VelloScene>>,
    lottie: Single<&ViewVisibility, With<VelloLottieHandle>>,
    svg: Single<&ViewVisibility, With<VelloSvg2d>>,
    text: Single<&ViewVisibility, With<VelloTextSection>>,
) {
    let visible_status = format!(
        "{{\n  scene: {},\n  lottie: {},\n  svg: {},\n  text: {}\n}}",
        scene.get(),
        lottie.get(),
        svg.get(),
        text.get()
    );

    println!("{visible_status}");
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));
}

fn load_view_culling(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn(VelloScene::new())
        .insert(RightLeft)
        .insert(Aabb::from_min_max(
            Vec3::new(-50.0, -50.0, 0.0),
            Vec3::new(50.0, 50.0, 0.0),
        ));

    // You can also use `VelloLottieBundle`
    commands
        .spawn(VelloLottieHandle(
            asset_server.load("embedded://view_culling/assets/lottie/Tiger.json"),
        ))
        .insert(Transform::from_scale(Vec3::splat(0.2)))
        .insert(LeftRight)
        .insert(Aabb::default());

    commands
        .spawn(VelloSvg2d(
            asset_server.load("embedded://view_culling/assets/svg/fountain.svg"),
        ))
        .insert(Transform::from_scale(Vec3::splat(1.0)))
        .insert(DownUp)
        .insert(Aabb::default());

    commands
        .spawn((
            VelloTextSection {
                value: "View culled text".to_string(),
                style: VelloTextStyle {
                    font_size: 24.0,
                    ..default()
                },
                ..default()
            },
            VelloTextAnchor::Center,
        ))
        .insert(UpDown)
        .insert(Aabb::default());
}

fn update_lottie_aabb(
    lottie_assets: Res<Assets<VelloLottie>>,
    mut lottie_q: Query<(&GlobalTransform, &VelloLottieHandle, &mut Aabb), Added<Aabb>>,
) {
    for (lottie_transform, handle, mut aabb) in &mut lottie_q {
        if let Some(vello_lottie) = lottie_assets.get(&handle.0) {
            let bb = vello_lottie.bb_in_world_space(lottie_transform);
            *aabb = Aabb::from_min_max(bb.min.extend(0.0), bb.max.extend(0.0));
        }
    }
}

fn update_svg_aabb(
    svg_assets: Res<Assets<VelloSvg>>,
    mut svg_q: Query<(&GlobalTransform, &VelloSvg2d, &mut Aabb), Added<Aabb>>,
) {
    for (svg_transform, handle, mut aabb) in &mut svg_q {
        if let Some(vello_svg) = svg_assets.get(&handle.0) {
            let bb = vello_svg.bb_in_world_space(svg_transform);
            *aabb = Aabb::from_min_max(bb.min.extend(0.0), bb.max.extend(0.0));
        }
    }
}

fn update_text_aabb(
    font_assets: Res<Assets<VelloFont>>,
    mut text_q: Query<(&GlobalTransform, &VelloTextSection, &mut Aabb), Added<Aabb>>,
) {
    for (text_transform, text_section, mut aabb) in &mut text_q {
        if let Some(font) = font_assets.get(&text_section.style.font) {
            let bb = text_section.bb_in_world_space(font, text_transform);
            *aabb = Aabb::from_min_max(bb.min.extend(0.0), bb.max.extend(0.0));
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
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

#[derive(Component)]
struct LeftRight;

const ANIMATION_SPEED: f32 = 0.5;

fn left_right(mut query: Query<&mut Transform, With<LeftRight>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::X * -800.0, Vec3::X * 800.0, sin_time);
    }
}

#[derive(Component)]
struct RightLeft;

fn right_left(mut query: Query<&mut Transform, With<RightLeft>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::X * 800.0, Vec3::X * -800.0, sin_time);
    }
}

#[derive(Component)]
struct UpDown;

fn up_down(mut query: Query<&mut Transform, With<UpDown>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::Y * -500.0, Vec3::Y * 500.0, sin_time);
    }
}

#[derive(Component)]
struct DownUp;

fn down_up(mut query: Query<&mut Transform, With<DownUp>>, time: Res<Time>) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5) * ANIMATION_SPEED;
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::lerp(Vec3::Y * 500.0, Vec3::Y * -500.0, sin_time);
    }
}

fn draw_aabb_gizmos(aabb_q: Query<(&Aabb, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (aabb, gtransform) in &aabb_q {
        let aabb_min = aabb.min();
        let aabb_max = aabb.max();
        gizmos.rect_2d(
            Isometry2d::new(
                gtransform.translation().xy(),
                Rot2::radians(gtransform.rotation().to_scaled_axis().z),
            ),
            Vec2::new(aabb_max.x - aabb_min.x, aabb_max.y - aabb_min.y) * gtransform.scale().xy(),
            Color::WHITE,
        );
    }
}
