use super::render;
use crate::render::extract::VelloExtractStep;
use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSystems},
};

pub struct SceneIntegrationPlugin;

impl Plugin for SceneIntegrationPlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                ExtractSchedule,
                (render::extract_world_scenes, render::extract_ui_scenes)
                    .in_set(VelloExtractStep::ExtractAssets),
            )
            .add_systems(
                Render,
                render::prepare_scene_affines.in_set(RenderSystems::Prepare),
            );
    }
}
