use super::{SkipScaling, VelloEntityCountData, VelloFrameProfileData};
use crate::prelude::*;
use bevy::math::Affine3A;
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    render::{
        Extract, MainWorld, camera::ExtractedCamera, extract_component::ExtractComponent,
        sync_world::TemporaryRenderEntity,
    },
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum VelloExtractStep {
    // Extract renderable types, e.g. SVG, Lottie, Text, Scenes
    ExtractAssets,
    // Synchronize frame data
    SyncData,
}

#[inline]
fn decompose_mat2(mat: Mat2) -> (f32, Vec2) {
    let x = mat.x_axis;
    let y = mat.y_axis;

    let scale_x = x.length();
    let scale_y = y.length();

    let angle = x.y.atan2(x.x);

    (angle, Vec2::new(scale_x, scale_y))
}

pub fn ui_to_global(ui: &UiGlobalTransform) -> GlobalTransform {
    let (angle, scale) = decompose_mat2(ui.matrix2);

    let rotation = Quat::from_rotation_z(angle);

    let scale = Vec3::new(scale.x, scale.y, 1.0);

    GlobalTransform::from_xyz(ui.translation.x, ui.translation.y, 0.0)
        * GlobalTransform::from_rotation(rotation)
        * GlobalTransform::from_scale(scale)
}

#[derive(Component, Clone)]
pub struct ExtractedVelloScene {
    pub scene: VelloScene,
    pub transform: GlobalTransform,
    pub ui_node: Option<ComputedNode>,
    pub screen_space: Option<VelloScreenSpace>,
    pub skip_scaling: Option<SkipScaling>,
    pub z_index: Option<ZIndex>,
}

pub fn extract_scenes(
    mut commands: Commands,
    query_views: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<Camera2d>, With<VelloView>),
    >,
    query_scenes: Extract<
        Query<
            (
                &VelloScene,
                &UiGlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
                Option<&VelloScreenSpace>,
                Option<&SkipScaling>,
                Option<&ZIndex>,
            ),
            Without<SkipEncoding>,
        >,
    >,
    mut frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_scenes = 0;

    // Sort cameras by rendering order
    let mut views: Vec<_> = query_views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        scene,
        transform,
        view_visibility,
        inherited_visibility,
        ui_node,
        render_layers,
        screen_space,
        skip_scaling,
        z_index,
    ) in query_scenes.iter()
    {
        // Skip if visibility conditions are not met
        if !view_visibility.get() || !inherited_visibility.get() {
            continue;
        }

        // Check if any camera renders this asset
        let asset_render_layers = render_layers.unwrap_or_default();
        if views.iter().any(|(_, camera_layers)| {
            asset_render_layers.intersects(camera_layers.unwrap_or_default())
        }) {
            commands
                .spawn(ExtractedVelloScene {
                    transform: ui_to_global(transform),
                    scene: scene.clone(),
                    ui_node: ui_node.cloned(),
                    screen_space: screen_space.cloned(),
                    skip_scaling: skip_scaling.cloned(),
                    z_index: z_index.cloned(),
                })
                .insert(TemporaryRenderEntity);
            n_scenes += 1;
        }
    }

    frame_data.n_scenes = n_scenes;
}

/// Synchronize the entity count data back to the main world.
pub fn sync_entity_count(render_data: Res<VelloEntityCountData>, mut world: ResMut<MainWorld>) {
    let mut main_world_data = world.get_resource_mut::<VelloEntityCountData>().unwrap();
    *main_world_data = render_data.clone();
}

/// Synchronize the frame profile data back to the main world.
pub fn sync_frame_profile(render_data: Res<VelloFrameProfileData>, mut world: ResMut<MainWorld>) {
    let mut main_world_data = world.get_resource_mut::<VelloFrameProfileData>().unwrap();
    *main_world_data = render_data.clone();
}

/// A screenspace render target. We use a resizable fullscreen quad.
#[derive(Component, Default)]
pub struct SSRenderTarget(pub Handle<Image>);

impl ExtractComponent for SSRenderTarget {
    type QueryData = &'static SSRenderTarget;

    type QueryFilter = ();

    type Out = Self;

    fn extract_component(
        ss_render_target: bevy::ecs::query::QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self> {
        Some(Self(ss_render_target.0.clone()))
    }
}
