use std::cmp::Reverse;

use bevy::{
    math::FloatOrd,
    picking::{
        PickSet,
        backend::{HitData, PointerHits},
        pointer::{PointerId, PointerLocation},
    },
    prelude::*,
};
use vello::kurbo::Shape;

use crate::VelloScreenSpace;

use super::{VelloScreenScale, VelloView, VelloWorldScale};

pub struct VelloPickingPlugin;

impl Plugin for VelloPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, vello_scene_picking.in_set(PickSet::Backend));
    }
}

/// Enables picking with a Vello bezier path.
#[derive(Default, Clone, Component)]
pub struct VelloPickingShape {
    pub bez_path: vello::kurbo::BezPath,
    pub affine: vello::kurbo::Affine,
}

fn vello_scene_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    camera: Single<(Entity, &Camera, &GlobalTransform, &Projection), With<VelloView>>,

    bez_paths_query: Query<(
        Entity,
        &VelloPickingShape,
        &GlobalTransform,
        Option<&Pickable>,
        Option<&VelloScreenSpace>,
    )>,

    mut output: EventWriter<PointerHits>,
    world_scale: Res<VelloWorldScale>,
    screen_scale: Res<VelloScreenScale>,
) {
    let (cam_entity, camera, cam_transform, cam_ortho) = *camera;

    if let Projection::Orthographic(cam_ortho) = cam_ortho {
        let mut sorted_bez_paths: Vec<_> = bez_paths_query
            .iter()
            .filter_map(
                |(entity, bez_path, transform, picking_behavior, screen_space)| {
                    if !transform.affine().is_nan() {
                        Some((entity, bez_path, transform, picking_behavior, screen_space))
                    } else {
                        None
                    }
                },
            )
            .collect();

        sorted_bez_paths.sort_by_key(|x| Reverse(FloatOrd(x.2.translation().z)));

        let world_scale_matrix = Mat4::from_scale_rotation_translation(
            Vec2::splat(world_scale.0).extend(1.0),
            Quat::IDENTITY,
            Vec3::ZERO,
        );

        let world_cam_transform = GlobalTransform::from(Transform::from_matrix(
            cam_transform.affine() * world_scale_matrix.inverse(),
        ));

        let screen_scale_matrix = Mat4::from_scale_rotation_translation(
            Vec3::splat(screen_scale.0),
            Quat::IDENTITY,
            Vec3::ZERO,
        );

        let screen_space_transform =
            GlobalTransform::from(Transform::from_matrix(screen_scale_matrix.inverse()));

        for (pointer, location) in pointers.iter() {
            let maybe_location = location.location();
            if maybe_location.is_none() {
                continue;
            }
            let pointer_location = maybe_location.unwrap();

            let mut blocked = false;

            let viewport_pos = camera
                .logical_viewport_rect()
                .map(|v| v.min)
                .unwrap_or_default();

            let pos_in_viewport = pointer_location.position - viewport_pos;

            let Ok(cursor_ray_world) = camera.viewport_to_world(cam_transform, pos_in_viewport)
            else {
                continue;
            };

            let cursor_ray_len = cam_ortho.far - cam_ortho.near;
            let cursor_ray_end =
                cursor_ray_world.origin + *cursor_ray_world.direction * cursor_ray_len;

            let picks: Vec<(Entity, HitData)> = sorted_bez_paths
                .iter()
                .copied()
                .filter_map(
                    |(
                        entity,
                        picking_shape,
                        picking_shape_transform,
                        picking_behavior,
                        screen_space,
                    )| {
                        if blocked {
                            return None;
                        }

                        let cam_transform = if screen_space.is_some() {
                            screen_space_transform
                        } else {
                            world_cam_transform
                        };

                        let world_to_bez_path = picking_shape_transform.affine().inverse();

                        let cursor_start =
                            world_to_bez_path.transform_point3(cursor_ray_world.origin);
                        let cursor_end = world_to_bez_path.transform_point3(cursor_ray_end);

                        let lerp_factor = f32::inverse_lerp(cursor_start.z, cursor_end.z, 0.0);
                        let cursor_pos_in_bez_path = cursor_start.lerp(cursor_end, lerp_factor);

                        let is_cursor_in_bez_path = picking_shape.bez_path.contains(
                            vello::kurbo::Point::new(
                                cursor_pos_in_bez_path.x as f64,
                                -cursor_pos_in_bez_path.y as f64,
                            ) + picking_shape.affine.inverse().translation(),
                        );

                        blocked = is_cursor_in_bez_path
                            && picking_behavior
                                .map(|p| p.should_block_lower)
                                .unwrap_or(true);

                        is_cursor_in_bez_path.then(|| {
                            let hit_pos_world = picking_shape_transform
                                .transform_point(pos_in_viewport.extend(0.0));
                            let hit_pos_cam = cam_transform
                                .affine()
                                .inverse()
                                .transform_point3(hit_pos_world);

                            let depth = -cam_ortho.near - hit_pos_cam.z;
                            (
                                entity,
                                HitData::new(
                                    cam_entity,
                                    depth,
                                    Some(hit_pos_world),
                                    Some(*picking_shape_transform.back()),
                                ),
                            )
                        })
                    },
                )
                .collect();

            let order = camera.order as f32;
            output.write(PointerHits::new(*pointer, picks, order));
        }
    }
}
