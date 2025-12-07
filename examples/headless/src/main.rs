use bevy::{
    camera::{RenderTarget, Viewport},
    core_pipeline::core_2d::graph::Core2d,
    diagnostic::FrameCount,
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    },
};
use bevy_vello::{VelloPlugin, prelude::*, vello::wgpu::TextureFormat};

#[derive(Resource)]
struct ScreenshotTarget(Handle<Image>);

fn main() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::render::RenderPlugin::default(),
        bevy::image::ImagePlugin::default(),
        bevy::camera::CameraPlugin,
        bevy::core_pipeline::CorePipelinePlugin,
        bevy::mesh::MeshPlugin,
        bevy::window::WindowPlugin::default(),
        bevy::sprite::SpritePlugin,
        bevy::sprite_render::SpriteRenderPlugin,
        bevy::text::TextPlugin,
    ))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, (setup_camera, setup_vector_graphics))
    .add_systems(Update, screenshot.run_if(|f: Res<FrameCount>| f.0 >= 1))
    .run();
}

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create image render target
    let image = Image::new_target_texture(100, 100, TextureFormat::Rgba8UnormSrgb);
    let image_handle = images.add(image);

    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                physical_size: UVec2 { x: 100, y: 100 },
                ..Default::default()
            }),
            target: RenderTarget::Image(image_handle.clone().into()),
            clear_color: ClearColorConfig::None,
            ..default()
        },
        CameraRenderGraph::new(Core2d),
        VelloView,
    ));

    // Store the image handle for screenshot
    commands.insert_resource(ScreenshotTarget(image_handle));
}

fn setup_vector_graphics(mut commands: Commands) {
    // Draw a circle
    let mut scene = VelloScene2d::new();
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([0.0, 1.0, 0.0, 1.0]),
        None,
        &kurbo::Circle::new((0.0, 0.0), 50.0),
    );
    commands.spawn(scene);
}

fn screenshot(mut commands: Commands, target: Res<ScreenshotTarget>) {
    let path = "./screenshot.png";
    commands
        .spawn(Screenshot::image(target.0.clone()))
        .observe(save_to_disk(path))
        .observe(exit_system);
}

fn exit_system(_trigger: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
