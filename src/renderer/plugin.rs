use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin, render_asset::RenderAssetPlugin,
        renderer::RenderDevice, Render, RenderApp, RenderSet,
    },
};
use vello::{Renderer, RendererOptions};

use super::{extract, prepare, render, VelatoRenderer, VelloRenderer};
use crate::{VelloFont, VelloVector};

pub struct VelloRenderPlugin;

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        render_app.insert_resource(VelatoRenderer(velato::Renderer::new()));

        render_app.add_systems(
            Render,
            prepare::prepare_vector_affines.in_set(RenderSet::Prepare),
        );
        render_app.add_systems(
            Render,
            prepare::prepare_vector_composition_edits.in_set(RenderSet::Prepare),
        );
        render_app.add_systems(
            Render,
            prepare::prepare_text_affines.in_set(RenderSet::Prepare),
        );
        render_app.add_systems(Render, render::render_scene.in_set(RenderSet::Render));

        app.add_plugin(ExtractComponentPlugin::<extract::ExtractedRenderVector>::default())
            .add_plugin(ExtractComponentPlugin::<extract::ExtractedRenderText>::default())
            .add_plugin(ExtractComponentPlugin::<extract::SSRenderTarget>::default())
            .add_plugin(RenderAssetPlugin::<VelloVector>::default())
            .add_plugin(RenderAssetPlugin::<VelloFont>::default())
            .add_systems(Update, extract::tag_vectors_for_render);
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        let device = render_app
            .world
            .get_resource::<RenderDevice>()
            .expect("bevy_vello: unable to get render device");

        render_app.insert_resource(VelloRenderer(
            Renderer::new(
                device.wgpu_device(),
                &RendererOptions {
                    surface_format: None,
                },
            )
            .unwrap(),
        ));
    }
}