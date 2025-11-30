use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::*,
};

use crate::render::{VelloEntityCountData, VelloFrameProfileData};

/// Adds Vello entity counting diagnostics to an App.
#[derive(Default)]
pub struct VelloEntityCountDiagnosticsPlugin;

impl Plugin for VelloEntityCountDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::diagnostic_system);

        // Scenes
        app.register_diagnostic(
            Diagnostic::new(Self::WORLD_SCENE_COUNT).with_suffix(" world scenes"),
        )
        .register_diagnostic(Diagnostic::new(Self::UI_SCENE_COUNT).with_suffix(" UI scenes"));
        // Text
        #[cfg(feature = "text")]
        app.register_diagnostic(
            Diagnostic::new(Self::WORLD_TEXT_COUNT).with_suffix(" world texts"),
        )
        .register_diagnostic(Diagnostic::new(Self::UI_TEXT_COUNT).with_suffix(" UI texts"));
        // Svg
        #[cfg(feature = "svg")]
        app.register_diagnostic(Diagnostic::new(Self::WORLD_SVG_COUNT).with_suffix(" world svgs"))
            .register_diagnostic(Diagnostic::new(Self::UI_SVG_COUNT).with_suffix(" UI svgs"));
        // Lottie
        #[cfg(feature = "lottie")]
        app.register_diagnostic(
            Diagnostic::new(Self::WORLD_LOTTIE_COUNT).with_suffix(" world lotties"),
        )
        .register_diagnostic(Diagnostic::new(Self::UI_LOTTIE_COUNT).with_suffix(" ui lotties"));
    }
}

impl VelloEntityCountDiagnosticsPlugin {
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

    fn diagnostic_system(mut diagnostics: Diagnostics, data: Res<VelloEntityCountData>) {
        diagnostics.add_measurement(&Self::WORLD_SCENE_COUNT, || data.n_world_scenes as f64);
        diagnostics.add_measurement(&Self::UI_SCENE_COUNT, || data.n_ui_scenes as f64);
        #[cfg(feature = "text")]
        {
            diagnostics.add_measurement(&Self::WORLD_TEXT_COUNT, || data.n_world_texts as f64);
            diagnostics.add_measurement(&Self::UI_TEXT_COUNT, || data.n_ui_texts as f64);
        }
        #[cfg(feature = "svg")]
        {
            diagnostics.add_measurement(&Self::WORLD_SVG_COUNT, || data.n_world_svgs as f64);
            diagnostics.add_measurement(&Self::UI_SVG_COUNT, || data.n_ui_svgs as f64);
        }
        #[cfg(feature = "lottie")]
        {
            diagnostics.add_measurement(&Self::WORLD_LOTTIE_COUNT, || data.n_world_lotties as f64);
            diagnostics.add_measurement(&Self::UI_LOTTIE_COUNT, || data.n_ui_lotties as f64);
        }
    }
}

/// Adds Vello frame profile diagnostics to an App.
#[derive(Default)]
pub struct VelloFrameProfileDiagnosticsPlugin;

impl Plugin for VelloFrameProfileDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::PATH_COUNT).with_suffix(" paths"))
            .register_diagnostic(
                Diagnostic::new(Self::PATH_SEGMENTS_COUNT).with_suffix(" path segments"),
            )
            .register_diagnostic(Diagnostic::new(Self::CLIPS_COUNT).with_suffix(" clips"))
            .register_diagnostic(Diagnostic::new(Self::OPEN_CLIPS_COUNT).with_suffix(" open clips"))
            .add_systems(Update, Self::diagnostic_system);
    }
}

impl VelloFrameProfileDiagnosticsPlugin {
    pub const PATH_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_paths");
    pub const PATH_SEGMENTS_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("vello_path_segments");
    pub const CLIPS_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_clips");
    pub const OPEN_CLIPS_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_open_clips");

    fn diagnostic_system(mut diagnostics: Diagnostics, data: Res<VelloFrameProfileData>) {
        diagnostics.add_measurement(&Self::PATH_COUNT, || data.n_paths as f64);
        diagnostics.add_measurement(&Self::PATH_SEGMENTS_COUNT, || data.n_path_segs as f64);
        diagnostics.add_measurement(&Self::CLIPS_COUNT, || data.n_clips as f64);
        diagnostics.add_measurement(&Self::OPEN_CLIPS_COUNT, || data.n_open_clips as f64);
    }
}
