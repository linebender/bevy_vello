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

#[cfg(feature = "text")]
use super::VelloFontsChanged;
use super::{VelloCanvasSettings, VelloRenderSettings, extract::VelloRenderTarget, systems};
use crate::{
    VelloView,
    render::{
        RT_SHADER_HANDLE, VelloCanvasMaterial, VelloEntityCountData, VelloFrameProfileData,
        VelloRenderQueue, VelloRenderer, diagnostics::VelloRenderDiagnosticsPlugin,
        extract::VelloExtractStep,
    },
};
#[cfg(feature = "text")]
use bevy::render::MainWorld;

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
            RT_SHADER_HANDLE,
            "../../shaders/vello_rendertarget.wgsl",
            Shader::from_wgsl
        );

        // Diagnostics
        app.add_plugins(VelloRenderDiagnosticsPlugin);

        #[cfg(feature = "text")]
        app.init_resource::<VelloFontsChanged>()
            .add_systems(PostUpdate, systems::detect_font_changes);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(self.render_settings.clone())
            .init_resource::<VelloEntityCountData>()
            .init_resource::<VelloFrameProfileData>()
            .init_resource::<VelloRenderQueue>();
        #[cfg(feature = "text")]
        render_app
            .init_resource::<VelloFontsChanged>()
            .init_resource::<crate::integrations::text::layout_cache::TextLayoutCache>()
            .add_systems(
                ExtractSchedule,
                extract_font_changed.after(VelloExtractStep::ExtractAssets),
            );
        render_app
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

/// Copies [`VelloFontsChanged`] from the main world into the render world.
#[cfg(feature = "text")]
fn extract_font_changed(main_world: Res<MainWorld>, mut font_changed: ResMut<VelloFontsChanged>) {
    if let Some(main_font_changed) = main_world.get_resource::<VelloFontsChanged>() {
        font_changed.0.clone_from(&main_font_changed.0);
    } else {
        font_changed.0.clear();
    }
}
