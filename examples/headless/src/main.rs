use bevy::{
    asset::embedded_asset,
    camera::RenderTarget,
    diagnostic::FrameCount,
    prelude::*,
    render::gpu_readback::{Readback, ReadbackComplete},
};
use bevy_vello::{VelloPlugin, prelude::*, vello::peniko::color::AlphaColor};

const SIZE: u32 = 900;

/// Stores the image handle used as the Vello render target so the readback
/// system can reference it later.
#[derive(Resource)]
struct CaptureImage(Handle<Image>);

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
        bevy::window::WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        },
        bevy::sprite::SpritePlugin,
        bevy::sprite_render::SpriteRenderPlugin,
        bevy::text::TextPlugin,
    ))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, (setup_camera, load_svg, load_text))
    .add_systems(Update, readback.run_if(|f: Res<FrameCount>| f.0 >= 3));
    embedded_asset!(app, "assets/Ghostscript_Tiger.svg");
    app.run();
}

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(VelloImage::new(SIZE, SIZE));
    commands.insert_resource(CaptureImage(image.clone()));
    commands.spawn((Camera2d, VelloView, RenderTarget::Image(image.into())));
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

fn readback(mut commands: Commands, image: Res<CaptureImage>, mut taken: Local<bool>) {
    if *taken {
        return;
    }
    *taken = true;
    commands
        .spawn(Readback::texture(image.0.clone()))
        .observe(save_and_exit);
}

fn save_and_exit(trigger: On<ReadbackComplete>, mut exit: MessageWriter<AppExit>) {
    // GPU textures may have row padding (aligned to 256 bytes). Strip it
    // so the raw RGBA bytes form a contiguous image buffer.
    let bytes_per_pixel = 4usize;
    let unpadded_bytes_per_row = SIZE as usize * bytes_per_pixel;
    let align = 256usize;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

    let mut pixels = Vec::with_capacity(unpadded_bytes_per_row * SIZE as usize);
    for row in 0..SIZE as usize {
        let start = row * padded_bytes_per_row;
        pixels.extend_from_slice(&trigger.data[start..start + unpadded_bytes_per_row]);
    }

    let img = image::RgbaImage::from_raw(SIZE, SIZE, pixels)
        .expect("Failed to create image from readback data");
    img.save("./screenshot.png")
        .expect("Failed to save screenshot");
    info!("Screenshot saved to ./screenshot.png");
    exit.write(AppExit::Success);
}
