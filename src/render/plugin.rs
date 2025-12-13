use bevy::{
    asset::load_internal_asset,
    camera::{CameraUpdateSystems, visibility::VisibilitySystems},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems, extract_component::ExtractComponentPlugin,
        renderer::RenderDevice,
    },
    sprite_render::Material2dPlugin,
};

use super::{VelloCanvasSettings, VelloRenderSettings, extract::VelloRenderTarget, systems};
use crate::{
    VelloView,
    render::{
        SSRT_SHADER_HANDLE, VelloCanvasMaterial, VelloEntityCountData, VelloFrameProfileData,
        VelloRenderQueue, VelloRenderer, diagnostics::VelloRenderDiagnosticsPlugin,
        extract::VelloExtractStep,
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

        // Diagnostics
        app.add_plugins(VelloRenderDiagnosticsPlugin);

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
                (
                    VelloExtractStep::ExtractAssets,
                    VelloExtractStep::RunDiagnostics,
                )
                    .chain()
                    .after(VisibilitySystems::CheckVisibility),
            )
            .add_systems(
                Render,
                (systems::sort_render_items, systems::render_frame)
                    .chain()
                    .in_set(RenderSystems::Render)
                    .run_if(resource_exists::<RenderDevice>),
            )
            .add_systems(
                Render,
                systems::render_settings_change_detection.in_set(RenderSystems::Cleanup),
            );

        app.add_plugins(ExtractComponentPlugin::<VelloView>::default());

        app.insert_resource(self.canvas_settings.clone())
            .add_plugins((
                Material2dPlugin::<VelloCanvasMaterial>::default(),
                ExtractComponentPlugin::<VelloRenderTarget>::default(),
            ))
            .add_systems(Startup, systems::setup_rendertarget)
            .add_systems(
                PostUpdate,
                (systems::resize_rendertargets.after(CameraUpdateSystems),),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<VelloRenderer>();
    }
}
