use bevy::{prelude::*, render::extract_component::ExtractComponent};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum VelloExtractStep {
    // Extract renderable types, e.g. SVG, Lottie, Text, Scenes
    ExtractAssets,
    // Measure frame
    RunDiagnostics,
}

/// A screenspace render target. We use a resizable fullscreen quad.
#[derive(Component, Default, Clone, ExtractComponent)]
pub struct VelloRenderTarget(pub Handle<Image>);
