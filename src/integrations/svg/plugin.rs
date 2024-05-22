use super::asset_loader::VelloSvgLoader;
use bevy::prelude::*;

pub struct SvgIntegrationPlugin;

impl Plugin for SvgIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloSvgLoader>();
    }
}
