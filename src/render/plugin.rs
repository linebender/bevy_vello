use bevy::{
    asset::load_internal_asset,
    camera::{CameraUpdateSystems, visibility::VisibilitySystems},
    prelude::*,
    render::{
        MainWorld, Render, RenderApp, RenderSystems, extract_component::ExtractComponentPlugin,
        renderer::RenderDevice,
    },
    sprite_render::Material2dPlugin,
    transform::TransformSystems,
};

use super::{
    VelloCanvasSettings, VelloFontChanged, VelloRenderSettings, VelloSceneDirty,
    extract::VelloRenderTarget, systems,
};
use crate::{
    VelloView,
    render::{
        RT_SHADER_HANDLE, VelloCanvasMaterial, VelloEntityCountData, VelloFrameProfileData,
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
            RT_SHADER_HANDLE,
            "../../shaders/vello_rendertarget.wgsl",
            Shader::from_wgsl
        );

        // Diagnostics
        app.add_plugins(VelloRenderDiagnosticsPlugin);

        // Dirty tracking resources in main world
        // VelloRenderSettings must be in the main world because
        // detect_vello_scene_changes reads it (runs in PostUpdate).
        app.insert_resource(self.render_settings.clone())
            .init_resource::<VelloSceneDirty>()
            .init_resource::<VelloFontChanged>()
            .add_systems(
                PostUpdate,
                systems::detect_vello_scene_changes
                    .after(TransformSystems::Propagate)
                    .after(VisibilitySystems::CheckVisibility),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(self.render_settings.clone())
            .init_resource::<VelloEntityCountData>()
            .init_resource::<VelloFrameProfileData>()
            .init_resource::<VelloRenderQueue>()
            .init_resource::<VelloSceneDirty>()
            .init_resource::<VelloFontChanged>();
        #[cfg(feature = "text")]
        render_app.init_resource::<crate::integrations::text::layout_cache::TextLayoutCache>();
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
                ExtractSchedule,
                extract_dirty_tracking.after(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                Render,
                (systems::sort_render_items, systems::render_frame)
                    .chain()
                    .in_set(RenderSystems::Render)
                    .run_if(resource_exists::<RenderDevice>)
                    .run_if(|dirty: Res<VelloSceneDirty>| dirty.0),
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

/// Copies [`VelloSceneDirty`] and [`VelloFontChanged`] from the main world
/// into the render world each frame.
fn extract_dirty_tracking(
    main_world: Res<MainWorld>,
    mut dirty: ResMut<VelloSceneDirty>,
    mut font_changed: ResMut<VelloFontChanged>,
) {
    if let Some(main_dirty) = main_world.get_resource::<VelloSceneDirty>() {
        dirty.0 = main_dirty.0;
    }
    if let Some(main_font_changed) = main_world.get_resource::<VelloFontChanged>() {
        font_changed.0 = main_font_changed.0;
    }
}
