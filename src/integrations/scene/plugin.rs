use super::render;
use crate::render::extract::VelloExtractStep;
use bevy::{prelude::*, render::RenderApp};

pub struct SceneIntegrationPlugin;

impl Plugin for SceneIntegrationPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "picking")]
        app.add_plugins(crate::picking::WorldPickingPlugin::<super::VelloScene2d>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            ExtractSchedule,
            (render::extract_world_scenes, render::extract_ui_scenes)
                .in_set(VelloExtractStep::ExtractAssets),
        );
    }
}
