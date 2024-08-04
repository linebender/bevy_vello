use super::{
    extract::{self, ExtractedPixelScale, SSRenderTarget},
    prepare, systems,
};
use crate::{
    render::SSRT_SHADER_HANDLE, VelloAsset, VelloCanvasMaterial, VelloFont, VelloScene,
    VelloTextSection,
};
use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_asset::RenderAssetPlugin,
        renderer::RenderDevice,
        view::{check_visibility, RenderLayers, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
    sprite::Material2dPlugin,
};

pub struct VelloRenderPlugin;

#[derive(Resource, Default, Clone, Debug)]
pub struct VelloRenderSettings {
    /// The render layer that will be used for the vello canvas mesh.
    pub canvas_render_layers: RenderLayers,
}

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
        render_app.add_systems(ExtractSchedule, extract::extract_svg_assets);
        #[cfg(feature = "lottie")]
        render_app
            .init_resource::<super::VelatoRenderer>()
            .add_systems(ExtractSchedule, extract::extract_lottie_assets);

        render_app
            .insert_resource(ExtractedPixelScale(1.0))
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_pixel_scale.in_set(RenderSet::ExtractCommands),
                    extract::extract_scenes,
                    extract::extract_text,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_asset_affines,
                    prepare::prepare_scene_affines,
                    prepare::prepare_text_affines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                systems::render_frame
                    .in_set(RenderSet::Render)
                    .run_if(resource_exists::<RenderDevice>),
            );

        app.add_plugins((
            Material2dPlugin::<VelloCanvasMaterial>::default(),
            ExtractComponentPlugin::<SSRenderTarget>::default(),
            RenderAssetPlugin::<VelloFont>::default(),
        ))
        .add_systems(Startup, systems::setup_ss_rendertarget)
        .add_systems(
            Update,
            (
                systems::resize_rendertargets,
                systems::hide_when_empty,
                systems::settings_change_detection,
            ),
        )
        .add_systems(
            PostUpdate,
            check_visibility::<
                Or<(
                    With<VelloScene>,
                    With<Handle<VelloAsset>>,
                    With<VelloTextSection>,
                )>,
            >
                .in_set(VisibilitySystems::CheckVisibility),
        );
    }
}
