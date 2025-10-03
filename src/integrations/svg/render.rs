use bevy::{
    prelude::*,
    render::{
        Extract,
        camera::ExtractedCamera,
        sync_world::TemporaryRenderEntity,
        view::{ExtractedView, RenderLayers},
    },
};
use kurbo::Affine;

use super::{
    VelloSvgAnchor,
    asset::{VelloSvg, VelloSvgHandle},
};
use crate::{
    VelloScreenSpace,
    prelude::*,
    render::{
        SkipScaling, VelloEntityCountData, VelloScreenScale, VelloWorldScale,
        prepare::{PrepareRenderInstance, PreparedAffine, PreparedTransform},
    },
};

#[derive(Component, Clone)]
pub struct ExtractedVelloSvg {
    pub asset: VelloSvg,
    pub asset_anchor: VelloSvgAnchor,
    pub transform: GlobalTransform,
    pub ui_node: Option<ComputedNode>,
    pub alpha: f32,
    pub screen_space: Option<VelloScreenSpace>,
    pub skip_scaling: Option<SkipScaling>,
    pub z_index: Option<ZIndex>,
}

pub fn extract_svg_assets(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_vectors: Extract<
        Query<
            (
                &VelloSvgHandle,
                &VelloSvgAnchor,
                &GlobalTransform,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&VelloScreenSpace>,
                Option<&SkipScaling>,
                Option<&ZIndex>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    assets: Extract<Res<Assets<VelloSvg>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_svgs = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        asset_handle,
        asset_anchor,
        transform,
        ui_node,
        render_layers,
        view_visibility,
        inherited_visibility,
        screen_space,
        skip_scaling,
        z_index,
    ) in query_vectors.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }
        // Skip if asset isn't loaded.
        let Some(asset) = assets.get(asset_handle.id()) else {
            continue;
        };

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloSvg {
                    asset: asset.to_owned(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    ui_node: ui_node.cloned(),
                    alpha: asset.alpha,
                    screen_space: screen_space.cloned(),
                    skip_scaling: skip_scaling.cloned(),
                    z_index: z_index.cloned(),
                })
                .insert(TemporaryRenderEntity);
            n_svgs += 1;
        }
    }

    frame_data.n_svgs = n_svgs;
}

pub fn prepare_asset_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView), With<Camera2d>>,
    mut render_entities: Query<(Entity, &ExtractedVelloSvg)>,
    world_scale: Res<VelloWorldScale>,
    screen_scale: Res<VelloScreenScale>,
) {
    for (camera, view) in views.iter() {
        let viewport_size: UVec2 = camera.physical_viewport_size.unwrap();
        for (entity, render_entity) in render_entities.iter_mut() {
            // Prepare render data needed for the subsequent render system
            let final_transform = render_entity.final_transform();
            let affine = render_entity.scene_affine(
                view,
                *final_transform,
                viewport_size,
                world_scale.0,
                screen_scale.0,
            );
            commands.entity(entity).insert((affine, final_transform));
        }
    }
}

impl PrepareRenderInstance for ExtractedVelloSvg {
    fn final_transform(&self) -> PreparedTransform {
        PreparedTransform(self.asset_anchor.compute(
            self.asset.width,
            self.asset.height,
            &self.transform,
        ))
    }

    fn scene_affine(
        &self,
        view: &ExtractedView,
        world_transform: GlobalTransform,
        viewport_size: UVec2,
        world_scale: f32,
        screen_scale: f32,
    ) -> PreparedAffine {
        let mut local_center_matrix = self.asset.local_transform_center.to_matrix().inverse();
        let is_scaled = self.skip_scaling.is_none();

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
        let transform: [f64; 6] = if let Some(node) = self.ui_node {
            let mut model_mat = world_transform.to_matrix();

            // Fill the bevy_ui Node with the asset size
            let asset_size = Vec2::new(self.asset.width, self.asset.height);
            let fill_scale = node.size() / asset_size;
            let scale_factor = fill_scale.x.min(fill_scale.y); // Maintain aspect ratio
            let scale_fact_mat = Mat4::from_scale(Vec3::new(scale_factor, scale_factor, 1.0));
            model_mat *= scale_fact_mat;

            if is_scaled {
                let scale_mat = Mat4::from_scale(Vec3::new(screen_scale, screen_scale, 1.0));
                model_mat *= scale_mat;
            }

            let raw_transform = model_mat * local_center_matrix;
            let transform = raw_transform.to_cols_array();
            [
                transform[0] as f64,  // a // scale_x
                transform[1] as f64,  // b // skew_y
                transform[4] as f64,  // c // skew_x
                transform[5] as f64,  // d // scale_y
                transform[12] as f64, // e // translate_x
                transform[13] as f64, // f // translate_y
            ]
        } else if self.screen_space.is_some() {
            let mut model_matrix = world_transform.to_matrix();

            if is_scaled {
                let scale_mat = Mat4::from_scale(Vec3::new(screen_scale, screen_scale, 1.0));
                model_matrix *= scale_mat;
            }

            let raw_transform = model_matrix * local_center_matrix;
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
                let scale_mat = Mat4::from_scale(Vec3::new(world_scale, world_scale, 1.0));
                model_matrix *= scale_mat;
            }

            // Flip Y-axis to center with Bevy's y-up world coordinate space
            local_center_matrix.w_axis.y *= -1.0;
            model_matrix *= local_center_matrix;

            // Flip Y-axis to match Vello's y-down coordinate space
            model_matrix.w_axis.y *= -1.0;

            let (projection_mat, view_mat) = {
                let mut view_mat = view.world_from_view.to_matrix();

                // Flip Y-axis to match Vello's y-down coordinate space
                view_mat.w_axis.y *= -1.0;

                (view.clip_from_view, view_mat)
            };
            let view_proj_matrix = projection_mat * view_mat.inverse();

            let (pixels_x, pixels_y) = (viewport_size.x as f32, viewport_size.y as f32);
            let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
                [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .transpose();

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

        PreparedAffine(Affine::new(transform))
    }
}
