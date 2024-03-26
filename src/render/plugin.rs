use super::extract::{self, ExtractedPixelScale, SSRenderTarget};
use super::{prepare, systems, BevyVelloRenderer, LottieRenderer};
use crate::render::extract::ExtractedRenderText;
use crate::render::SSRT_SHADER_HANDLE;
use crate::{VelloCanvasMaterial, VelloFont};
use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::render_asset::RenderAssetPlugin;
use bevy::render::renderer::RenderDevice;
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::sprite::Material2dPlugin;
use vello::{AaSupport, Renderer, RendererOptions};

pub struct VelloRenderPlugin;

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "../../shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(LottieRenderer::default())
            .insert_resource(ExtractedPixelScale(1.0))
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_pixel_scale.in_set(RenderSet::ExtractCommands),
                    extract::asset_instances,
                    extract::scene_instances,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_vector_affines,
                    prepare::prepare_scene_affines,
                    prepare::prepare_text_affines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, systems::render_scene.in_set(RenderSet::Render));

        app.add_plugins((
            Material2dPlugin::<VelloCanvasMaterial>::default(),
            ExtractComponentPlugin::<ExtractedRenderText>::default(),
            ExtractComponentPlugin::<SSRenderTarget>::default(),
            RenderAssetPlugin::<VelloFont>::default(),
        ))
        .add_systems(Startup, systems::setup_ss_rendertarget)
        .add_systems(
            Update,
            (systems::resize_rendertargets, systems::clear_when_empty),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        let device = render_app
            .world
            .get_resource::<RenderDevice>()
            .expect("bevy_vello: unable to get render device");

        render_app.insert_non_send_resource(BevyVelloRenderer(
            Renderer::new(
                device.wgpu_device(),
                RendererOptions {
                    surface_format: None,
                    use_cpu: false,
                    antialiasing_support: AaSupport {
                        area: true,
                        msaa8: false,
                        msaa16: false,
                    },
                    num_init_threads: None,
                },
            )
            .unwrap(),
        ));
    }
}
