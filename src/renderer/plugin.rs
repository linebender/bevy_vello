use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin, render_asset::RenderAssetPlugin, RenderApp,
        RenderSet,
    },
};

use super::{extract, prepare, render, VelatoRenderer, VelloRenderer};
use crate::{VelloFont, VelloVector};

pub struct VelloRenderPlugin;

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        render_app.init_resource::<VelloRenderer>();
        render_app.insert_resource(VelatoRenderer(velato::Renderer::new()));

        render_app.add_system(prepare::prepare_vector_affines.in_set(RenderSet::Prepare));
        render_app.add_system(prepare::prepare_vector_composition_edits.in_set(RenderSet::Prepare));
        render_app.add_system(prepare::prepare_text_affines.in_set(RenderSet::Prepare));
        render_app.add_system(render::render_scene.in_set(RenderSet::Render));

        app.add_plugin(ExtractComponentPlugin::<extract::ExtractedRenderVector>::default())
            .add_plugin(ExtractComponentPlugin::<extract::ExtractedRenderText>::default())
            .add_plugin(ExtractComponentPlugin::<extract::SSRenderTarget>::default())
            .add_plugin(RenderAssetPlugin::<VelloVector>::default())
            .add_plugin(RenderAssetPlugin::<VelloFont>::default())
            .add_system(extract::tag_vectors_for_render);
    }
}
