use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_asset::RenderAssetPlugin, renderer::RenderDevice, Render,
        RenderApp, RenderSet,
    },
};
use vello::{AaSupport, Renderer, RendererOptions};

use super::{
    extract::{self, ExtractedPixelScale},
    prepare, render, BevyVelloRenderer, LottieRenderer,
};
use crate::VelloFont;

pub struct VelloRenderPlugin;

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(LottieRenderer::default())
            .insert_resource(ExtractedPixelScale(1.0))
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_pixel_scale
                        .in_set(RenderSet::ExtractCommands),
                    extract::vector_instances,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_vector_affines,
                    prepare::prepare_text_affines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                render::render_scene.in_set(RenderSet::Render),
            );

        app.add_plugins((
            ExtractComponentPlugin::<extract::ExtractedRenderText>::default(),
            ExtractComponentPlugin::<extract::SSRenderTarget>::default(),
            RenderAssetPlugin::<VelloFont>::default(),
        ));
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
                    timestamp_period: 0.0,
                    use_cpu: false,
                    antialiasing_support: AaSupport {
                        area: true,
                        msaa8: false,
                        msaa16: false,
                    },
                },
            )
            .unwrap(),
        ));
    }
}
