use crate::{
    VelloFont, VelloRenderSettings,
    render::{VelloCanvasSettings, VelloRenderPlugin},
    text::VelloFontLoader,
};
use bevy::{prelude::*, render::view::RenderLayers};
use vello::AaConfig;

#[derive(Clone)]
pub struct VelloPlugin {
    /// The render layers that will be used for the Vello canvas mesh.
    pub canvas_render_layers: RenderLayers,

    /// Use CPU instead of GPU
    pub use_cpu: bool,

    /// Which antialiasing strategy to use
    pub antialiasing: AaConfig,
}

impl Default for VelloPlugin {
    fn default() -> Self {
        let default_canvas_settings = VelloCanvasSettings::default();
        let default_render_settings = VelloRenderSettings::default();
        Self {
            canvas_render_layers: default_canvas_settings.render_layers,
            use_cpu: default_render_settings.use_cpu,
            antialiasing: default_render_settings.antialiasing,
        }
    }
}

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VelloRenderPlugin {
            canvas_settings: VelloCanvasSettings {
                render_layers: self.canvas_render_layers.clone(),
            },
            render_settings: VelloRenderSettings {
                use_cpu: self.use_cpu,
                antialiasing: self.antialiasing,
            },
        })
        .init_asset::<VelloFont>()
        .init_asset_loader::<VelloFontLoader>();
        #[cfg(feature = "svg")]
        app.add_plugins(crate::integrations::svg::SvgIntegrationPlugin);
        #[cfg(feature = "lottie")]
        app.add_plugins(crate::integrations::lottie::LottieIntegrationPlugin);
        #[cfg(feature = "default_font")]
        {
            let mut fonts = app
                .world_mut()
                .get_resource_mut::<Assets<VelloFont>>()
                .unwrap();

            fonts.insert(
                Handle::default().id(),
                crate::text::load_into_font_context(bevy::text::DEFAULT_FONT_DATA.to_vec()),
            );
        }
    }
}
