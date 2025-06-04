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
        app.register_diagnostic(Diagnostic::new(Self::SCENE_COUNT).with_suffix(" scenes"))
            .add_systems(Update, Self::diagnostic_system);
        #[cfg(feature = "text")]
        app.register_diagnostic(Diagnostic::new(Self::TEXT_COUNT).with_suffix(" texts"));
        #[cfg(feature = "svg")]
        app.register_diagnostic(Diagnostic::new(Self::SVG_COUNT).with_suffix(" svgs"));
        #[cfg(feature = "lottie")]
        app.register_diagnostic(Diagnostic::new(Self::LOTTIE_COUNT).with_suffix(" lotties"));
    }
}

impl VelloEntityCountDiagnosticsPlugin {
    pub const SCENE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_scenes");
    #[cfg(feature = "text")]
    pub const TEXT_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_texts");
    #[cfg(feature = "svg")]
    pub const SVG_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_svgs");
    #[cfg(feature = "lottie")]
    pub const LOTTIE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_lotties");

    fn diagnostic_system(mut diagnostics: Diagnostics, data: Res<VelloEntityCountData>) {
        diagnostics.add_measurement(&Self::SCENE_COUNT, || data.n_scenes as f64);
        #[cfg(feature = "text")]
        diagnostics.add_measurement(&Self::TEXT_COUNT, || data.n_texts as f64);
        #[cfg(feature = "svg")]
        diagnostics.add_measurement(&Self::SVG_COUNT, || data.n_svgs as f64);
        #[cfg(feature = "lottie")]
        diagnostics.add_measurement(&Self::LOTTIE_COUNT, || data.n_lotties as f64);
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
