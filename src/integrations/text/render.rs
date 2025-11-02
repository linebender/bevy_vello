use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    render::{
        Extract, camera::ExtractedCamera, sync_world::TemporaryRenderEntity, view::ExtractedView,
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
    pub skip_scaling: Option<SkipScaling>,
    pub z_index: Option<ZIndex>,
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
                Option<&ZIndex>,
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
        skip_scaling,
        z_index,
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
                    skip_scaling: skip_scaling.cloned(),
                    z_index: z_index.cloned(),
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
    let screen_scale_matrix = Mat4::from_scale(Vec3::new(screen_scale.0, screen_scale.0, 1.0));
    let world_scale_matrix = Mat4::from_scale(Vec3::new(world_scale.0, world_scale.0, 1.0));

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
            let is_scaled = render_entity.skip_scaling.is_none();

            // A transposed (flipped over its diagonal) PostScript matrix
            // | a c e |
            // | b d f |
            // | 0 0 1 |
            //
            // Components
            // | scale_x skew_x translate_x |
            // | skew_y scale_y translate_y |
            // | skew_z skew_z scale_z |
            //
            // rotate (z)
            // | cos(θ) -sin(θ) translate_x |
            // | sin(θ) cos(θ) translate_y |
            // | skew_z skew_z scale_z |
            //
            // The order of operations is important, as it affects the final transformation matrix.
            //
            // Order of operations:
            // 1. Scale
            // 2. Rotate
            // 3. Translate
            let transform: [f64; 6] =
                if render_entity.ui_node.is_some() || render_entity.screen_space.is_some() {
                    let mut model_matrix = world_transform.to_matrix();

                    if is_scaled {
                        model_matrix *= screen_scale_matrix;
                    }

                    let raw_transform = model_matrix;
                    let transform = raw_transform.to_cols_array();

                    [
                        transform[0] as f64,  // a // scale_x
                        transform[1] as f64,  // b // skew_y
                        transform[4] as f64,  // c // skew_x
                        transform[5] as f64,  // d // scale_y
                        transform[12] as f64, // e // translate_x
                        transform[13] as f64, // f // translate_y
                    ]
                } else {
                    let mut model_matrix = world_transform.to_matrix();

                    if is_scaled {
                        model_matrix *= world_scale_matrix;
                    }

                    // Flip Y-axis to match Vello's y-down coordinate space
                    model_matrix.w_axis.y *= -1.0;

                    let (projection_mat, view_mat) = {
                        let mut view_mat = view.world_from_view.to_matrix();

                        // Flip Y-axis to match Vello's y-down coordinate space
                        view_mat.w_axis.y *= -1.0;

                        (view.clip_from_view, view_mat)
                    };
                    let view_proj_matrix = projection_mat * view_mat.inverse();

                    let raw_transform = ndc_to_pixels_matrix * view_proj_matrix * model_matrix;
                    let transform = raw_transform.to_cols_array();

                    // Negate skew_x and skew_y to match rotation of the Bevy's y-up world
                    [
                        transform[0] as f64,  // a // scale_x
                        -transform[1] as f64, // b // skew_y
                        -transform[4] as f64, // c // skew_x
                        transform[5] as f64,  // d // scale_y
                        transform[12] as f64, // e // translate_x
                        transform[13] as f64, // f // translate_y
                    ]
                };

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}
