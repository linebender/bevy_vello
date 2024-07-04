use super::extract::{self, ExtractedPixelScale, SSRenderTarget};
use super::{prepare, systems};
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

pub struct VelloRenderPlugin;

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "../../shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        #[cfg(feature = "svg")]
        render_app.add_systems(ExtractSchedule, extract::extract_svg_instances);
        #[cfg(feature = "lottie")]
        render_app
            .init_resource::<super::VelatoRenderer>()
            .add_systems(ExtractSchedule, extract::extract_lottie_instances);

        render_app
            .insert_resource(ExtractedPixelScale(1.0))
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_pixel_scale.in_set(RenderSet::ExtractCommands),
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
            .add_systems(
                Render,
                systems::render_scene
                    .in_set(RenderSet::Render)
                    .run_if(resource_exists::<RenderDevice>),
            );

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
}
