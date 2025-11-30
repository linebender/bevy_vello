use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    render::{
        Extract, camera::ExtractedCamera, sync_world::TemporaryRenderEntity, view::ExtractedView,
    },
};
use kurbo::Affine;

use super::{
    VelloSvgAnchor,
    asset::{VelloSvg, VelloSvgHandle},
};
use crate::{
    prelude::*,
    render::{
        SkipScaling, VelloEntityCountData, VelloScreenScale, VelloWorldScale,
        prepare::PreparedAffine,
    },
};

#[derive(Component, Clone)]
pub struct ExtractedWorldVelloSvg {
    pub asset: VelloSvg,
    pub asset_anchor: VelloSvgAnchor,
    pub transform: GlobalTransform,
    pub alpha: f32,
    pub render_space: VelloRenderSpace,
    pub skip_scaling: Option<SkipScaling>,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloSvg {
    pub asset: VelloSvg,
    pub asset_anchor: VelloSvgAnchor,
    pub ui_transform: UiGlobalTransform,
    pub ui_node: ComputedNode,
    pub alpha: f32,
    pub skip_scaling: Option<SkipScaling>,
}

pub fn extract_world_svg_assets(
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
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
                &VelloRenderSpace,
                Option<&SkipScaling>,
            ),
            (Without<SkipEncoding>, Without<Node>),
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
        render_layers,
        view_visibility,
        inherited_visibility,
        render_space,
        skip_scaling,
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
                .spawn(ExtractedWorldVelloSvg {
                    asset: asset.to_owned(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    alpha: asset.alpha,
                    render_space: *render_space,
                    skip_scaling: skip_scaling.cloned(),
                })
                .insert(TemporaryRenderEntity);
            n_svgs += 1;
        }
    }

    frame_data.n_world_svgs = n_svgs;
}

pub fn extract_ui_svg_assets(
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
                &UiGlobalTransform,
                &ComputedNode,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&SkipScaling>,
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
        ui_transform,
        ui_node,
        render_layers,
        view_visibility,
        inherited_visibility,
        skip_scaling,
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
                .spawn(ExtractedUiVelloSvg {
                    asset: asset.to_owned(),
                    asset_anchor: *asset_anchor,
                    ui_transform: *ui_transform,
                    ui_node: *ui_node,
                    alpha: asset.alpha,
                    skip_scaling: skip_scaling.cloned(),
                })
                .insert(TemporaryRenderEntity);
            n_svgs += 1;
        }
    }

    frame_data.n_ui_svgs = n_svgs;
}

pub fn prepare_asset_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView), (With<Camera2d>, With<VelloView>)>,
    render_entities: Query<(Entity, &ExtractedWorldVelloSvg)>,
    render_ui_entities: Query<(Entity, &ExtractedUiVelloSvg)>,
    world_scale: Res<VelloWorldScale>,
    screen_scale: Res<VelloScreenScale>,
) {
    let screen_scale_matrix = Mat4::from_scale(Vec3::new(screen_scale.0, screen_scale.0, 1.0));
    let world_scale_matrix = Mat4::from_scale(Vec3::new(world_scale.0, world_scale.0, 1.0));

    for (camera, view) in views.iter() {
        let viewport_size: UVec2 = camera.physical_viewport_size.unwrap();
        let (pixels_x, pixels_y) = (viewport_size.x as f32, viewport_size.y as f32);
        let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
            [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
            [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
        .transpose();

        // Process UI entities
        for (entity, render_entity) in render_ui_entities.iter() {
            let ui_transform = render_entity.ui_transform;
            let ui_node = render_entity.ui_node;
            let needs_scaling = render_entity.skip_scaling.is_none();

            let local_center_matrix = render_entity
                .asset
                .local_transform_center
                .to_matrix()
                .inverse();

            let transform: [f64; 6] = {
                // Convert UiGlobalTransform to Mat4
                let mat2 = ui_transform.matrix2;
                let translation = ui_transform.translation;
                let mut model_mat = Mat4::from_cols_array_2d(&[
                    [mat2.x_axis.x, mat2.x_axis.y, 0.0, 0.0],
                    [mat2.y_axis.x, mat2.y_axis.y, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [translation.x, translation.y, 0.0, 1.0],
                ]);

                // Fill the bevy_ui Node with the asset size
                let asset_size = Vec2::new(render_entity.asset.width, render_entity.asset.height);
                let fill_scale = ui_node.size() / asset_size;
                let scale_factor = fill_scale.x.min(fill_scale.y); // Maintain aspect ratio
                let scale_fact_mat = Mat4::from_scale(Vec3::new(scale_factor, scale_factor, 1.0));
                model_mat *= scale_fact_mat;

                if needs_scaling {
                    model_mat *= screen_scale_matrix;
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
            };

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }

        // Process World entities
        for (entity, render_entity) in render_entities.iter() {
            let world_transform = render_entity.transform;
            let needs_scaling = render_entity.skip_scaling.is_none();

            // Compute final transform with anchor
            let final_transform = render_entity.asset_anchor.compute(
                render_entity.asset.width,
                render_entity.asset.height,
                render_entity.render_space,
                &world_transform,
            );

            let mut local_center_matrix = render_entity
                .asset
                .local_transform_center
                .to_matrix()
                .inverse();

            let transform: [f64; 6] = if render_entity.render_space == VelloRenderSpace::Screen {
                let mut model_matrix = final_transform.to_matrix();

                if needs_scaling {
                    model_matrix *= screen_scale_matrix;
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
                // VelloRenderSpace::World
                let mut model_matrix = final_transform.to_matrix();

                if needs_scaling {
                    model_matrix *= world_scale_matrix;
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
