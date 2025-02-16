use super::{
    extract::{self, ExtractedPixelScale, SSRenderTarget},
    prepare, systems, VelloCanvasSettings, VelloRenderSettings,
};
use crate::{
    render::{
        extract::VelloExtractStep, VelloCanvasMaterial, VelloFrameData, VelloRenderQueue,
        VelloRenderer, SSRT_SHADER_HANDLE,
    },
    VelloFont, VelloScene, VelloTextSection, VelloView,
};
use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_asset::RenderAssetPlugin,
        renderer::RenderDevice,
        view::{check_visibility, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
    sprite::Material2dPlugin,
};

#[derive(Default)]
pub struct VelloRenderPlugin {
    /// Settings used for the canvas
    pub canvas_settings: VelloCanvasSettings,

    /// Settings used for rendering with Vello
    pub render_settings: VelloRenderSettings,
}

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "../../shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<VelloFrameData>()
            .init_resource::<VelloFrameData>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(self.render_settings.clone())
            .insert_resource(ExtractedPixelScale(1.0))
            .init_resource::<VelloFrameData>()
            .init_resource::<VelloRenderQueue>()
            .configure_sets(
                ExtractSchedule,
                (VelloExtractStep::ExtractAssets, VelloExtractStep::SyncData).chain(),
            )
            .add_systems(
                ExtractSchedule,
                extract::extract_pixel_scale.in_set(RenderSet::ExtractCommands),
            )
            .add_systems(
                ExtractSchedule,
                (extract::extract_scenes, extract::extract_text)
                    .in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                ExtractSchedule,
                extract::sync_frame_data.in_set(VelloExtractStep::SyncData),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_scene_affines,
                    prepare::prepare_text_affines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                (systems::sort_render_items, systems::render_frame)
                    .chain()
                    .in_set(RenderSet::Render)
                    .run_if(resource_exists::<RenderDevice>),
            )
            .add_systems(
                Render,
                systems::render_settings_change_detection.in_set(RenderSet::Cleanup),
            );

        app.add_plugins(ExtractComponentPlugin::<VelloView>::default());

        app.insert_resource(self.canvas_settings.clone())
            .add_plugins((
                Material2dPlugin::<VelloCanvasMaterial>::default(),
                ExtractComponentPlugin::<SSRenderTarget>::default(),
                RenderAssetPlugin::<VelloFont>::default(),
            ))
            .add_systems(Startup, systems::setup_ss_rendertarget)
            .add_systems(
                Update,
                (systems::resize_rendertargets, systems::hide_when_empty),
            )
            .add_systems(
                PostUpdate,
                check_visibility::<Or<(With<VelloScene>, With<VelloTextSection>)>>
                    .in_set(VisibilitySystems::CheckVisibility),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<VelloRenderer>();
    }
}
