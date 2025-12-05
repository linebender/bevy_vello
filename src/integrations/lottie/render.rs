use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    render::{
        Extract, camera::ExtractedCamera, sync_world::TemporaryRenderEntity, view::ExtractedView,
    },
};
use vello::kurbo::Affine;

use super::{Playhead, Theme, VelloLottieAnchor, asset::VelloLottie};
use crate::integrations::lottie::{UiVelloLottie, VelloLottie2d};
use crate::{
    SkipEncoding,
    render::{VelloEntityCountData, VelloView, prepare::PreparedAffine},
};

#[derive(Component, Clone)]
pub struct ExtractedVelloLottie2d {
    pub asset: VelloLottie,
    pub asset_anchor: VelloLottieAnchor,
    pub transform: GlobalTransform,
    pub alpha: f32,
    pub theme: Option<Theme>,
    pub playhead: f64,
}

#[derive(Component, Clone)]
pub struct ExtractedUiVelloLottie {
    pub asset: VelloLottie,
    pub ui_transform: UiGlobalTransform,
    pub alpha: f32,
    pub theme: Option<Theme>,
    pub playhead: f64,
    pub ui_node: ComputedNode,
}

pub fn extract_world_lottie_assets(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_vectors: Extract<
        Query<
            (
                &VelloLottie2d,
                &VelloLottieAnchor,
                &GlobalTransform,
                &Playhead,
                Option<&Theme>,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
            ),
            (Without<SkipEncoding>, Without<Node>),
        >,
    >,
    assets: Extract<Res<Assets<VelloLottie>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_lotties = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        asset_handle,
        asset_anchor,
        transform,
        playhead,
        theme,
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
        for (_, _) in views.iter().filter(|(_, camera_layers)| {
            // Does this camera can see this?
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloLottie2d {
                    asset: asset.clone(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    theme: theme.cloned(),
                    playhead: playhead.frame(),
                    alpha: asset.alpha,
                })
                .insert(TemporaryRenderEntity);
            n_lotties += 1;
        }
    }

    frame_data.n_world_lotties = n_lotties;
}

pub fn extract_ui_lottie_assets(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_vectors: Extract<
        Query<
            (
                &UiVelloLottie,
                &UiGlobalTransform,
                &Playhead,
                Option<&Theme>,
                &ComputedNode,
                Option<&RenderLayers>,
                &InheritedVisibility,
            ),
            Without<SkipEncoding>,
        >,
    >,
    assets: Extract<Res<Assets<VelloLottie>>>,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_lotties = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        asset_handle,
        ui_transform,
        playhead,
        theme,
        ui_node,
        render_layers,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        // Skip if visibility conditions are not met.
        // UI does not check view visibility, only inherited visibility.
        if !inherited_visibility.get() {
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
                .spawn(ExtractedUiVelloLottie {
                    asset: asset.clone(),
                    ui_transform: *ui_transform,
                    theme: theme.cloned(),
                    playhead: playhead.frame(),
                    alpha: asset.alpha,
                    ui_node: *ui_node,
                })
                .insert(TemporaryRenderEntity);
            n_lotties += 1;
        }
    }

    frame_data.n_ui_lotties = n_lotties;
}

pub fn prepare_asset_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView), (With<Camera2d>, With<VelloView>)>,
    render_entities: Query<(Entity, &ExtractedVelloLottie2d)>,
    render_ui_entities: Query<(Entity, &ExtractedUiVelloLottie)>,
) {
    for (camera, view) in views.iter() {
        // Render UI
        for (entity, render_entity) in render_ui_entities.iter() {
            let ui_transform = render_entity.ui_transform;

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
            let transform: [f64; 6] = {
                // Convert UiGlobalTransform to Mat4
                let mat2 = ui_transform.matrix2;
                let translation = ui_transform.translation;
                let model_matrix = Mat4::from_cols_array_2d(&[
                    [mat2.x_axis.x, mat2.x_axis.y, 0.0, 0.0],
                    [mat2.y_axis.x, mat2.y_axis.y, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [translation.x, translation.y, 0.0, 1.0],
                ]);
                let local_center_matrix = Transform::from_translation(Vec3 {
                    x: render_entity.asset.width / 2.0,
                    y: render_entity.asset.height / 2.0,
                    z: 0.0,
                })
                .to_matrix()
                .inverse();
                // Fill the bevy_ui Node with the asset size
                let aspect_fill_matrix = {
                    let asset_size =
                        Vec2::new(render_entity.asset.width, render_entity.asset.height);
                    let fill_scale = render_entity.ui_node.size() / asset_size;
                    let scale_factor = fill_scale.x.min(fill_scale.y); // Maintain aspect ratio
                    Mat4::from_scale(Vec3::new(scale_factor, scale_factor, 1.0))
                };

                // Transform chain: ui_transform (in logical px) → aspect_fill → local_center
                let raw_transform = model_matrix * aspect_fill_matrix * local_center_matrix;
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

        // Render World
        for (entity, render_entity) in render_entities.iter() {
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
            let transform: [f64; 6] = {
                // Get the base world transform
                let world_transform = render_entity.transform.compute_transform();
                let Transform {
                    translation,
                    rotation,
                    scale,
                } = world_transform;

                // Calculate anchor offset in local space (Vello's top-left origin)
                let anchor_local = match render_entity.asset_anchor {
                    VelloLottieAnchor::TopLeft => Vec3::ZERO,
                    VelloLottieAnchor::Left => {
                        Vec3::new(0.0, render_entity.asset.height / 2.0, 0.0)
                    }
                    VelloLottieAnchor::BottomLeft => {
                        Vec3::new(0.0, render_entity.asset.height, 0.0)
                    }
                    VelloLottieAnchor::Top => Vec3::new(render_entity.asset.width / 2.0, 0.0, 0.0),
                    VelloLottieAnchor::Center => Vec3::new(
                        render_entity.asset.width / 2.0,
                        render_entity.asset.height / 2.0,
                        0.0,
                    ),
                    VelloLottieAnchor::Bottom => Vec3::new(
                        render_entity.asset.width / 2.0,
                        render_entity.asset.height,
                        0.0,
                    ),
                    VelloLottieAnchor::TopRight => Vec3::new(render_entity.asset.width, 0.0, 0.0),
                    VelloLottieAnchor::Right => Vec3::new(
                        render_entity.asset.width,
                        render_entity.asset.height / 2.0,
                        0.0,
                    ),
                    VelloLottieAnchor::BottomRight => {
                        Vec3::new(render_entity.asset.width, render_entity.asset.height, 0.0)
                    }
                };

                let mut anchor_matrix = Mat4::from_translation(-anchor_local);
                // The anchor offset is in Vello's y-down coordinate space, but needs to be applied
                // in the transform chain that operates in Bevy's y-up space. This y-flip compensates
                // for the coordinate system difference before the final model_matrix y-flip (below).
                anchor_matrix.w_axis.y *= -1.0;

                let ndc_to_pixels_matrix = {
                    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
                    let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);
                    Mat4::from_cols_array_2d(&[
                        [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                        [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ])
                    .transpose()
                };
                let view_proj_matrix = {
                    let mut view_mat = view.world_from_view.to_matrix();
                    // Flip Y-axis to match Vello's y-down coordinate space
                    view_mat.w_axis.y *= -1.0;
                    let proj_mat = view.clip_from_view;
                    proj_mat * view_mat.inverse()
                };

                // Build the model matrix with proper anchor handling
                let translation_matrix = Mat4::from_translation(translation);
                let rotation_matrix = Mat4::from_quat(rotation);
                let scale_matrix = Mat4::from_scale(scale);

                // Build model matrix: translate → rotate → scale → camera_scale → anchor offset (including local_center)
                let mut model_matrix =
                    translation_matrix * rotation_matrix * scale_matrix * anchor_matrix;

                // Flip Y-axis to match Vello's y-down coordinate space
                model_matrix.w_axis.y *= -1.0;

                // Transform chain: world → anchor (with local_center) → y-flip → view → projection → NDC → pixels
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
