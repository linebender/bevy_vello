use crate::{
    assets::VelloAssetLoader,
    debug::DebugVisualizationsPlugin,
    player::LottiePlayerPlugin,
    render::{VelloRenderPlugin, SSRT_SHADER_HANDLE},
    text::VelloFontLoader,
    VelloAsset, VelloCanvasMaterial, VelloFont,
};
use bevy::{asset::load_internal_asset, prelude::*, sprite::Material2dPlugin};

pub struct VelloPlugin;

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VelloRenderPlugin)
            .add_plugins((LottiePlayerPlugin, DebugVisualizationsPlugin))
            .init_asset::<VelloAsset>()
            .init_asset_loader::<VelloAssetLoader>()
            .init_asset::<VelloFont>()
            .init_asset_loader::<VelloFontLoader>();
    }
}
