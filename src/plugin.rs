use crate::{
    assets::VelloAssetLoader, debug::DebugVisualizationsPlugin,
    player::LottiePlayerPlugin, render::VelloRenderPlugin,
    text::VelloFontLoader, VelloAsset, VelloFont,
};
use bevy::prelude::*;

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
