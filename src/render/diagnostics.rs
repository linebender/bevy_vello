use crate::render::extract::VelloExtractStep;
use crate::render::{VelloEntityCountData, VelloFrameProfileData};
use bevy::ecs::system::RunSystemOnce;
use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::*,
    render::{MainWorld, RenderApp},
};

pub const WORLD_SCENE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_world_scenes");
pub const UI_SCENE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_ui_scenes");
#[cfg(feature = "text")]
pub const WORLD_TEXT_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_world_texts");
#[cfg(feature = "text")]
pub const UI_TEXT_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_ui_texts");
#[cfg(feature = "svg")]
pub const WORLD_SVG_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_world_svgs");
#[cfg(feature = "svg")]
pub const UI_SVG_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_ui_svgs");
#[cfg(feature = "lottie")]
pub const WORLD_LOTTIE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_world_lotties");
#[cfg(feature = "lottie")]
pub const UI_LOTTIE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_ui_lotties");

pub const PATH_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_paths");
pub const PATH_SEGMENTS_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_path_segments");
pub const CLIPS_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_clips");
pub const OPEN_CLIPS_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_open_clips");

/// Adds Vello render diagnostics reporting.
#[derive(Default)]
pub(crate) struct VelloRenderDiagnosticsPlugin;

impl Plugin for VelloRenderDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(PATH_COUNT).with_suffix(" paths"))
            .register_diagnostic(Diagnostic::new(PATH_SEGMENTS_COUNT).with_suffix(" path segments"))
            .register_diagnostic(Diagnostic::new(CLIPS_COUNT).with_suffix(" clips"))
            .register_diagnostic(Diagnostic::new(OPEN_CLIPS_COUNT).with_suffix(" open clips"));

        // Scenes
        app.register_diagnostic(Diagnostic::new(WORLD_SCENE_COUNT).with_suffix(" world scenes"))
            .register_diagnostic(Diagnostic::new(UI_SCENE_COUNT).with_suffix(" UI scenes"));
        // Text
        #[cfg(feature = "text")]
        app.register_diagnostic(Diagnostic::new(WORLD_TEXT_COUNT).with_suffix(" world texts"))
            .register_diagnostic(Diagnostic::new(UI_TEXT_COUNT).with_suffix(" UI texts"));
        // Svg
        #[cfg(feature = "svg")]
        app.register_diagnostic(Diagnostic::new(WORLD_SVG_COUNT).with_suffix(" world svgs"))
            .register_diagnostic(Diagnostic::new(UI_SVG_COUNT).with_suffix(" UI svgs"));
        // Lottie
        #[cfg(feature = "lottie")]
        app.register_diagnostic(Diagnostic::new(WORLD_LOTTIE_COUNT).with_suffix(" world lotties"))
            .register_diagnostic(Diagnostic::new(UI_LOTTIE_COUNT).with_suffix(" ui lotties"));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<VelloFrameProfileData>()
            .init_resource::<VelloEntityCountData>()
            .add_systems(
                ExtractSchedule,
                (sync_entity_count, sync_frame_profile).in_set(VelloExtractStep::RunDiagnostics),
            );
    }
}

/// Measure the entity count.
fn sync_entity_count(render_data: Res<VelloEntityCountData>, mut main_world: ResMut<MainWorld>) {
    let n_world_scenes = render_data.n_world_scenes as f64;
    let n_ui_scenes = render_data.n_ui_scenes as f64;
    #[cfg(feature = "text")]
    let n_world_texts = render_data.n_world_texts as f64;
    #[cfg(feature = "text")]
    let n_ui_texts = render_data.n_ui_texts as f64;
    #[cfg(feature = "svg")]
    let n_world_svgs = render_data.n_world_svgs as f64;
    #[cfg(feature = "svg")]
    let n_ui_svgs = render_data.n_ui_svgs as f64;
    #[cfg(feature = "lottie")]
    let n_world_lotties = render_data.n_world_lotties as f64;
    #[cfg(feature = "lottie")]
    let n_ui_lotties = render_data.n_ui_lotties as f64;

    let result = main_world.run_system_once(move |mut diagnostics: Diagnostics| {
        diagnostics.add_measurement(&WORLD_SCENE_COUNT, || n_world_scenes);
        diagnostics.add_measurement(&UI_SCENE_COUNT, || n_ui_scenes);
        #[cfg(feature = "text")]
        {
            diagnostics.add_measurement(&WORLD_TEXT_COUNT, || n_world_texts);
            diagnostics.add_measurement(&UI_TEXT_COUNT, || n_ui_texts);
        }
        #[cfg(feature = "svg")]
        {
            diagnostics.add_measurement(&WORLD_SVG_COUNT, || n_world_svgs);
            diagnostics.add_measurement(&UI_SVG_COUNT, || n_ui_svgs);
        }
        #[cfg(feature = "lottie")]
        {
            diagnostics.add_measurement(&WORLD_LOTTIE_COUNT, || n_world_lotties);
            diagnostics.add_measurement(&UI_LOTTIE_COUNT, || n_ui_lotties);
        }
    });

    if let Err(e) = result {
        tracing::error!("Error recording vello frame measurements: {e}");
    }
}

/// Measure the frame profile.
fn sync_frame_profile(
    render_data: Res<VelloFrameProfileData>,
    mut main_world: ResMut<MainWorld>,
) -> Result {
    let n_paths = render_data.n_paths as f64;
    let n_path_segs = render_data.n_path_segs as f64;
    let n_clips = render_data.n_clips as f64;
    let n_open_clips = render_data.n_open_clips as f64;
    main_world
        .run_system_once(move |mut diagnostics: Diagnostics| {
            diagnostics.add_measurement(&PATH_COUNT, || n_paths);
            diagnostics.add_measurement(&PATH_SEGMENTS_COUNT, || n_path_segs);
            diagnostics.add_measurement(&CLIPS_COUNT, || n_clips);
            diagnostics.add_measurement(&OPEN_CLIPS_COUNT, || n_open_clips);
        })
        .unwrap_or_else(|e| {
            tracing::error!("Error recording vello frame measurements: {e}");
        });
    Ok(())
}
