use bevy::prelude::*;
use vello::AaConfig;

use crate::{VelloRenderSettings, render::VelloRenderPlugin};

#[derive(Clone)]
pub struct VelloPlugin {
    /// Use CPU instead of GPU
    pub use_cpu: bool,

    /// Which antialiasing strategy to use
    pub antialiasing: AaConfig,
}

impl Default for VelloPlugin {
    fn default() -> Self {
        let default_render_settings = VelloRenderSettings::default();
        Self {
            use_cpu: default_render_settings.use_cpu,
            antialiasing: default_render_settings.antialiasing,
        }
    }
}

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VelloRenderPlugin {
            render_settings: VelloRenderSettings {
                use_cpu: self.use_cpu,
                antialiasing: self.antialiasing,
            },
        });
        app.add_plugins(crate::integrations::scene::SceneIntegrationPlugin);
        #[cfg(feature = "svg")]
        app.add_plugins(crate::integrations::svg::SvgIntegrationPlugin);
        #[cfg(feature = "lottie")]
        app.add_plugins(crate::integrations::lottie::LottieIntegrationPlugin);
        #[cfg(feature = "text")]
        app.add_plugins(crate::integrations::text::VelloTextIntegrationPlugin);
    }
}
