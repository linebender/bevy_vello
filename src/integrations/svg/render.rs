use super::{
    asset::{VelloSvg, VelloSvgHandle},
    VelloSvgAnchor,
};
use crate::{
    prelude::*,
    render::{
        prepare::{PrepareRenderInstance, PreparedAffine, PreparedTransform},
        VelloEntityCountData,
    },
};
use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        sync_world::TemporaryRenderEntity,
        view::{ExtractedView, RenderLayers},
        Extract,
    },
};
use kurbo::Affine;

#[derive(Component, Clone)]
pub struct ExtractedVelloSvg {
    pub asset: VelloSvg,
    pub asset_anchor: VelloSvgAnchor,
    pub transform: GlobalTransform,
    pub ui_node: Option<ComputedNode>,
    pub alpha: f32,
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
) {
    for (camera, view) in views.iter() {
        let viewport_size: UVec2 = camera.physical_viewport_size.unwrap();
        for (entity, render_entity) in render_entities.iter_mut() {
            // Prepare render data needed for the subsequent render system
            let final_transform = render_entity.final_transform();
            let affine = render_entity.scene_affine(view, *final_transform, viewport_size);
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
    ) -> PreparedAffine {
        let local_center_matrix = self.asset.local_transform_center.compute_matrix().inverse();

        let raw_transform = if let Some(node) = self.ui_node {
            let mut model_matrix = world_transform.compute_matrix();

            let asset_size = Vec2::new(self.asset.width, self.asset.height);

            // Make the screen space vector instance sized to fill the
            // entire UI Node box if it's bundled with a Node
            let fill_scale = node.size() / asset_size;
            let scale_factor = fill_scale.x.min(fill_scale.y); // Maintain aspect ratio
            model_matrix.x_axis.x *= scale_factor;
            model_matrix.y_axis.y *= scale_factor;

            let mut local_center_matrix = local_center_matrix;
            local_center_matrix.w_axis.y *= -1.0;
            model_matrix * local_center_matrix
        } else {
            let local_matrix = local_center_matrix;

            let (pixels_x, pixels_y) = (viewport_size.x as f32, viewport_size.y as f32);
            let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
                [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .transpose();

            let mut model_matrix = world_transform.compute_matrix() * local_matrix;
            model_matrix.w_axis.y *= -1.0;

            let (projection_mat, view_mat) = {
                let mut view_mat = view.world_from_view.compute_matrix();
                view_mat.w_axis.y *= -1.0;

                (view.clip_from_view, view_mat)
            };

            let view_proj_matrix = projection_mat * view_mat.inverse();

            ndc_to_pixels_matrix * view_proj_matrix * model_matrix
        };

        let transform: [f32; 16] = raw_transform.to_cols_array();

        // | a c e |
        // | b d f |
        // | 0 0 1 |
        let transform: [f64; 6] = [
            transform[0] as f64,  // a
            -transform[1] as f64, // b
            -transform[4] as f64, // c
            transform[5] as f64,  // d
            transform[12] as f64, // e
            transform[13] as f64, // f
        ];

        PreparedAffine(Affine::new(transform))
    }
}
