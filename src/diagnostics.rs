use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::*,
};

use crate::render::VelloFrameData;

/// Adds vello diagnostics to an App, specifically asset counting and profiling.
#[derive(Default)]
pub struct VelloDiagnosticsPlugin;

impl Plugin for VelloDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::SCENE_COUNT))
            .register_diagnostic(Diagnostic::new(Self::TEXT_COUNT))
            .add_systems(Update, Self::diagnostic_system);

        #[cfg(feature = "svg")]
        app.register_diagnostic(Diagnostic::new(Self::SVG_COUNT));
        #[cfg(feature = "lottie")]
        app.register_diagnostic(Diagnostic::new(Self::LOTTIE_COUNT));
    }
}

impl VelloDiagnosticsPlugin {
    pub const SCENE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_scenes");
    pub const TEXT_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_texts");
    #[cfg(feature = "svg")]
    pub const SVG_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_svgs");
    #[cfg(feature = "lottie")]
    pub const LOTTIE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vello_lotties");

    fn diagnostic_system(mut diagnostics: Diagnostics, data: Res<VelloFrameData>) {
        diagnostics.add_measurement(&Self::SCENE_COUNT, || data.n_scenes as f64);
        diagnostics.add_measurement(&Self::TEXT_COUNT, || data.n_texts as f64);
        #[cfg(feature = "svg")]
        diagnostics.add_measurement(&Self::SVG_COUNT, || data.n_svgs as f64);
        #[cfg(feature = "lottie")]
        diagnostics.add_measurement(&Self::LOTTIE_COUNT, || data.n_lotties as f64);
    }
}
