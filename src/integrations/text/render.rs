use bevy::{
    prelude::*,
    render::{
        Extract,
        camera::ExtractedCamera,
        sync_world::TemporaryRenderEntity,
        view::{ExtractedView, RenderLayers},
    },
};
use vello::kurbo::Affine;

use super::{VelloFont, VelloTextAnchor, VelloTextSection};
use crate::{
    VelloScreenSpace,
    render::{
        SkipEncoding, SkipScaling, VelloEntityCountData, VelloScreenScale, VelloView,
        VelloWorldScale, prepare::PreparedAffine,
    },
};

#[derive(Component, Clone)]
pub struct ExtractedVelloText {
    pub text: VelloTextSection,
    pub text_anchor: VelloTextAnchor,
    pub transform: GlobalTransform,
    pub ui_node: Option<ComputedNode>,
    pub screen_space: Option<VelloScreenSpace>,
    pub no_scaling: Option<SkipScaling>,
}

pub fn extract_text(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &VelloTextSection,
                &VelloTextAnchor,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&RenderLayers>,
                Option<&ComputedNode>,
                Option<&VelloScreenSpace>,
                Option<&SkipScaling>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    fonts: Extract<Res<Assets<VelloFont>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_texts = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        text,
        text_anchor,
        transform,
        view_visibility,
        inherited_visibility,
        render_layers,
        ui_node,
        screen_space,
        no_scaling,
    ) in query_scenes.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }
        // Skip if font isn't loaded.
        let Some(_font) = fonts.get(text.style.font.id()) else {
            continue;
        };

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloText {
                    text: text.clone(),
                    text_anchor: *text_anchor,
                    transform: *transform,
                    ui_node: ui_node.cloned(),
                    screen_space: screen_space.cloned(),
                    no_scaling: no_scaling.cloned(),
                })
                .insert(TemporaryRenderEntity);
            n_texts += 1;
        }
    }

    frame_data.n_texts = n_texts;
}

pub fn prepare_text_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView), (With<Camera2d>, With<VelloView>)>,
    render_entities: Query<(Entity, &ExtractedVelloText)>,
    world_scale: Res<VelloWorldScale>,
    screen_scale: Res<VelloScreenScale>,
) {
    for (camera, view) in views.iter() {
        let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
        let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
        let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
            [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
            [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .transpose();

        for (entity, render_entity) in render_entities.iter() {
            let world_transform = render_entity.transform;

            // | scale_x sheer_x translate_x |
            // | sheer_y scale_y translate_y |
            // | sheer_z sheer_z scale_z |
            //
            // rotate (z)
            // | cos(θ) -sin(θ) translate_x |
            // | sin(θ) cos(θ) translate_y |
            // | sheer_z sheer_z scale_z |
            //
            // | a c e |
            // | b d f | => a transposed (flipped over its diagonal) PostScript matrix
            // | 0 0 1 |
            let transform: [f64; 6] = if let Some(_node) = render_entity.ui_node {
                let model_matrix = world_transform.compute_matrix();

                // Fill the bevy_ui Node with the asset size
                // let asset_size = Vec2::new(self.asset.width, self.asset.height);
                // let fill_scale = node.size() / asset_size;
                // Maintain aspect ratio
                // let scale_factor = fill_scale.x.min(fill_scale.y);
                // model_matrix.x_axis.x *= scale_factor;
                // model_matrix.y_axis.y *= scale_factor;
                //
                // if self.no_scaling.is_none() {
                //     model_matrix.x_axis.x *= screen_scale;
                //     model_matrix.y_axis.y *= screen_scale;
                // }

                let transform = model_matrix.to_cols_array();
                [
                    transform[0] as f64,  // a
                    transform[1] as f64,  // b
                    transform[4] as f64,  // c
                    transform[5] as f64,  // d
                    transform[12] as f64, // e
                    transform[13] as f64, // f
                ]
            } else if render_entity.screen_space.is_some() {
                let model_matrix = world_transform.compute_matrix();

                // if self.no_scaling.is_none() {
                //     model_matrix.x_axis.x *= screen_scale;
                //     model_matrix.y_axis.y *= screen_scale;
                // }

                let transform = model_matrix.to_cols_array();
                [
                    transform[0] as f64,  // a
                    transform[1] as f64,  // b
                    transform[4] as f64,  // c
                    transform[5] as f64,  // d
                    transform[12] as f64, // e
                    transform[13] as f64, // f
                ]
            } else {
                let mut model_matrix = world_transform.compute_matrix();
                model_matrix.w_axis.y *= -1.0;

                // if self.no_scaling.is_none() {
                //     model_matrix.x_axis.x *= world_scale;
                //     model_matrix.y_axis.y *= world_scale;
                // }

                let (projection_mat, view_mat) = {
                    let mut view_mat = view.world_from_view.compute_matrix();
                    view_mat.w_axis.y *= -1.0;
                    (view.clip_from_view, view_mat)
                };

                let view_proj_matrix = projection_mat * view_mat.inverse();
                let raw_transform = ndc_to_pixels_matrix * view_proj_matrix * model_matrix;
                let transform = raw_transform.to_cols_array();
                [
                    transform[0] as f64,  // a
                    -transform[1] as f64, // b
                    -transform[4] as f64, // c
                    transform[5] as f64,  // d
                    transform[12] as f64, // e
                    transform[13] as f64, // f
                ]
            };

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}
