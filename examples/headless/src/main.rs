use bevy::{
    asset::embedded_asset,
    camera::{RenderTarget, Viewport},
    core_pipeline::core_2d::graph::Core2d,
    diagnostic::FrameCount,
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    },
};
use bevy_vello::{
    VelloPlugin,
    prelude::*,
    vello::{peniko::color::AlphaColor, wgpu::TextureFormat},
};

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
    .add_systems(Startup, (setup_camera, load_svg, load_text))
    .add_systems(Update, screenshot.run_if(|f: Res<FrameCount>| f.0 >= 1));
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    app.run();
}

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create image render target
    let image = Image::new_target_texture(900, 900, TextureFormat::Rgba8UnormSrgb);
    let image_handle = images.add(image);

    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                physical_size: UVec2 { x: 900, y: 900 },
                ..Default::default()
            }),
            target: RenderTarget::Image(image_handle.clone().into()),
            ..default()
        },
        CameraRenderGraph::new(Core2d),
        VelloView,
    ));

    // Store the image handle for screenshot
    commands.insert_resource(ScreenshotTarget(image_handle));
}

fn load_svg(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(VelloSvg2d(
        asset_server.load("embedded://headless/assets/Ghostscript_Tiger.svg"),
    ));
}

fn load_text(mut commands: Commands) {
    commands.spawn((
        VelloText2d {
            value: "bevy_vello".to_string(),
            style: VelloTextStyle {
                brush: peniko::Brush::Solid(AlphaColor::from_rgb8(255, 0, 255)),
                font_size: 96.0,
                ..default()
            },
            ..default()
        },
        VelloTextAnchor::Center,
    ));
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
