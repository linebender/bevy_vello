use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        Render, RenderApp, RenderSet, extract_component::ExtractComponentPlugin,
        renderer::RenderDevice,
    },
    sprite::Material2dPlugin,
};

use super::{
    VelloCanvasSettings, VelloRenderSettings,
    extract::{self, SSRenderTarget},
    prepare, systems,
};
use crate::{
    VelloView,
    render::{
        SSRT_SHADER_HANDLE, VelloCanvasMaterial, VelloEntityCountData, VelloFrameProfileData,
        VelloRenderQueue, VelloRenderer, extract::VelloExtractStep,
    },
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

        app.register_type::<VelloEntityCountData>()
            .init_resource::<VelloEntityCountData>();
        app.register_type::<VelloFrameProfileData>()
            .init_resource::<VelloFrameProfileData>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(self.render_settings.clone())
            .init_resource::<VelloEntityCountData>()
            .init_resource::<VelloFrameProfileData>()
            .init_resource::<VelloRenderQueue>()
            .configure_sets(
                ExtractSchedule,
                (VelloExtractStep::ExtractAssets, VelloExtractStep::SyncData).chain(),
            )
            .add_systems(
                ExtractSchedule,
                extract::extract_scenes.in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                ExtractSchedule,
                (extract::sync_frame_profile, extract::sync_entity_count)
                    .in_set(VelloExtractStep::SyncData),
            )
            .add_systems(
                Render,
                prepare::prepare_scene_affines.in_set(RenderSet::Prepare),
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
            ))
            .add_systems(Startup, systems::setup_ss_rendertarget)
            .add_systems(
                Update,
                (systems::resize_rendertargets, systems::hide_when_empty),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<VelloRenderer>();
    }
}
