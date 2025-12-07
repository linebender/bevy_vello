use crate::render::VelloView;
use bevy::{
    camera::primitives::Aabb,
    picking::{
        PickingSystems,
        backend::{HitData, PointerHits},
        pointer::{PointerId, PointerLocation},
    },
    prelude::*,
    window::PrimaryWindow,
};
use std::marker::PhantomData;

#[derive(Default)]
pub struct WorldPickingPlugin<C: Component> {
    _type: PhantomData<C>,
}

impl<C: Component> Plugin for WorldPickingPlugin<C> {
    fn build(&self, app: &mut App) {
        debug!("Adding picking support for {}", std::any::type_name::<C>());
        app.add_systems(
            PreUpdate,
            update_aabb_hits::<C>.in_set(PickingSystems::Backend),
        );
    }
}

fn update_aabb_hits<C: Component>(
    primary_window: Single<Entity, With<PrimaryWindow>>,
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform), With<VelloView>>,
    aabb_query: Query<(Entity, &Aabb, &GlobalTransform, &Pickable), With<C>>,
    mut pointer_hits_writer: MessageWriter<PointerHits>,
) {
    for (pointer_id, pointer_location) in &pointers {
        let Some(ref location) = pointer_location.location else {
            continue;
        };

        // Find camera matching the pointer's render target
        let Some((cam_entity, camera, cam_transform)) = cameras.iter().find(|(_, cam, _)| {
            cam.target
                .normalize(Some(*primary_window))
                .is_some_and(|x| x == location.target)
        }) else {
            continue;
        };

        // Convert pointer position to world space ray
        let Ok(ray) = camera.viewport_to_world(cam_transform, location.position) else {
            continue;
        };

        let mut picks = Vec::new();
        let mut blocked = false;

        // Sort entities by distance for proper hit order
        let mut sorted_entities: Vec<_> = aabb_query.iter().collect();
        sorted_entities.sort_by(|(_, _, a, _), (_, _, b, _)| {
            b.translation()
                .z
                .partial_cmp(&a.translation().z)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (entity, aabb, transform, pickable) in sorted_entities {
            if blocked {
                continue;
            }

            // Transform ray to entity's local space
            let world_to_local = transform.affine().inverse();
            let local_ray_origin = world_to_local.transform_point3(ray.origin);
            let local_ray_direction = world_to_local.transform_vector3(*ray.direction);

            // Check ray-AABB intersection
            if intersects_aabb(local_ray_origin, local_ray_direction, aabb) {
                let hit_pos = ray.origin
                    + *ray.direction
                        * calculate_distance(ray.origin, *ray.direction, *transform, aabb);

                let hit_data = HitData::new(
                    cam_entity,
                    calculate_depth(hit_pos, cam_transform),
                    Some(hit_pos),
                    None,
                );

                picks.push((entity, hit_data));
                blocked = pickable.should_block_lower;
            }
        }

        if !picks.is_empty() {
            pointer_hits_writer.write(PointerHits::new(*pointer_id, picks, camera.order as f32));
        }
    }
}

/// Checks if a ray intersects with an AABB using slab method
fn intersects_aabb(ray_origin: Vec3, ray_direction: Vec3, aabb: &Aabb) -> bool {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for i in 0..3 {
        let origin = ray_origin[i];
        let dir = ray_direction[i];
        let min = aabb.min()[i];
        let max = aabb.max()[i];

        if dir.abs() < f32::EPSILON {
            // Ray is parallel to slab
            if origin < min || origin > max {
                return false;
            }
        } else {
            let t1 = (min - origin) / dir;
            let t2 = (max - origin) / dir;

            let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };

            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);

            if t_min > t_max {
                return false;
            }
        }
    }

    t_max >= 0.0
}

/// Calculates the distance along the ray to the AABB intersection point
fn calculate_distance(
    ray_origin: Vec3,
    ray_direction: Vec3,
    transform: GlobalTransform,
    aabb: &Aabb,
) -> f32 {
    // Transform ray to entity's local space
    let world_to_local = transform.affine().inverse();
    let local_ray_origin = world_to_local.transform_point3(ray_origin);
    let local_ray_direction = world_to_local.transform_vector3(ray_direction);

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for i in 0..3 {
        let origin = local_ray_origin[i];
        let dir = local_ray_direction[i];
        let min = aabb.min()[i];
        let max = aabb.max()[i];

        if dir.abs() > f32::EPSILON {
            let t1 = (min - origin) / dir;
            let t2 = (max - origin) / dir;

            let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };

            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);
        }
    }

    // Return the nearest intersection distance (use t_min if positive, else t_max)
    if t_min >= 0.0 { t_min } else { t_max }
}

/// Calculates the depth of a hit position in camera space
fn calculate_depth(hit_pos: Vec3, cam_transform: &GlobalTransform) -> f32 {
    let view_matrix = cam_transform.compute_transform().to_matrix().inverse();
    let hit_in_view = view_matrix.transform_point3(hit_pos);
    -hit_in_view.z
}
