use crate::{
    debug::DebugVisualizationsPlugin, render::VelloRenderPlugin, text::VelloFontLoader, VelloAsset,
    VelloFont,
};
use bevy::{asset::load_internal_binary_asset, prelude::*, render::view::RenderLayers};

#[derive(Default)]
pub struct VelloPlugin {
    pub canvas_render_layers: Option<RenderLayers>,
}

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VelloRenderPlugin {
            canvas_render_layers: self.canvas_render_layers.clone(),
        })
        .add_plugins(DebugVisualizationsPlugin)
        .init_asset::<VelloAsset>()
        .init_asset::<VelloFont>()
        .init_asset_loader::<VelloFontLoader>();
        #[cfg(feature = "svg")]
        app.add_plugins(crate::integrations::svg::SvgIntegrationPlugin);
        #[cfg(feature = "lottie")]
        app.add_plugins(crate::integrations::lottie::LottieIntegrationPlugin);
        #[cfg(feature = "experimental-dotLottie")]
        app.add_plugins(crate::integrations::dot_lottie::DotLottieIntegrationPlugin);
        #[cfg(feature = "default_font")]
        load_internal_binary_asset!(
            app,
            Handle::default(),
            "text/FiraMono-subset.ttf",
            |bytes: &[u8], _path: String| { VelloFont::new(bytes.to_vec()) }
        );
    }
}
