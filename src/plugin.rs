use crate::assets::VelloAssetLoader;
use crate::debug::DebugVisualizationsPlugin;
use crate::player::LottiePlayerPlugin;
use crate::render::VelloRenderPlugin;
use crate::text::VelloFontLoader;
use crate::{VelloAsset, VelloFont};
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
