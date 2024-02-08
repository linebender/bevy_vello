use crate::{
    font::VelloFont,
    player::LottiePlayerPlugin,
    renderer::VelloRenderPlugin,
    rendertarget::{self, SSRT_SHADER_HANDLE},
    VelloAsset, VelloAssetLoader, VelloFontLoader,
};
use bevy::{asset::load_internal_asset, prelude::*, sprite::Material2dPlugin};

pub struct VelloPlugin;

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "../shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(VelloRenderPlugin)
            .add_plugins((
                Material2dPlugin::<rendertarget::VelloCanvasMaterial>::default(),
                LottiePlayerPlugin,
                #[cfg(feature = "debug")]
                crate::debug::DebugVisualizationsPlugin,
            ))
            .init_asset::<VelloAsset>()
            .init_asset_loader::<VelloAssetLoader>()
            .init_asset::<VelloFont>()
            .init_asset_loader::<VelloFontLoader>()
            .add_systems(Startup, rendertarget::setup_ss_rendertarget)
            .add_systems(
                Update,
                (
                    rendertarget::resize_rendertargets,
                    rendertarget::clear_when_empty,
                ),
            );
    }
}
