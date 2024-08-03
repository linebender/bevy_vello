use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
};
use bevy_vello::{prelude::*, render::VelloRenderer, VelloPlugin};

#[derive(Component)]
pub struct VelloTarget(Handle<Image>);

impl ExtractComponent for VelloTarget {
    type QueryData = &'static VelloTarget;
    type QueryFilter = ();
    type Out = Self;
    fn extract_component(target: bevy::ecs::query::QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(Self(target.0.clone()))
    }
}

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, cube_rotator_system)
        .add_plugins(ExtractComponentPlugin::<VelloTarget>::default());

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app.add_systems(
        Render,
        render_texture
            .in_set(RenderSet::Render)
            .run_if(resource_exists::<RenderDevice>),
    );

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });
    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(4.0, 4.0, 4.0)),
            material: material_handle,
            transform: Transform::from_xyz(0.0, 0.0, 1.5)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 5.0)),
            ..default()
        },
        MainPassCube,
    ));
    // The main pass camera.
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(VelloTarget(image_handle));
}

fn render_texture(
    mut vello_renderer: Local<Option<VelloRenderer>>,
    target: Query<&VelloTarget>,
    device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    queue: Res<RenderQueue>,
    time: Res<Time>,
) {
    let renderer =
        vello_renderer.get_or_insert_with(|| VelloRenderer::from_device(device.wgpu_device()));
    let target = target.single();

    let mut scene = VelloScene::default();
    // Animate the scene
    let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let c = Vec3::lerp(
        Vec3::new(-1.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 1.0),
        sin_time + 0.5,
    );
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::translate((128.0, 128.0)),
        peniko::Color::rgb(c.x as f64, c.y as f64, c.z as f64),
        None,
        &kurbo::RoundedRect::new(0.0, 0.0, 256.0, 256.0, (sin_time as f64) * 128.0),
    );

    let gpu_image = gpu_images.get(&target.0).unwrap();
    let params = vello::RenderParams {
        base_color: vello::peniko::Color::WHITE,
        width: gpu_image.size.x,
        height: gpu_image.size.y,
        antialiasing_method: vello::AaConfig::Area,
    };
    renderer
        .render_to_texture(
            device.wgpu_device(),
            &queue,
            &scene,
            &gpu_image.texture_view,
            &params,
        )
        .unwrap();
}

/// Rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.0 * time.delta_seconds());
        transform.rotate_y(0.7 * time.delta_seconds());
    }
}
