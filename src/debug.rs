use bevy::prelude::*;

use crate::VelloVector;

pub struct DebugVisualizationsPlugin;

impl Plugin for DebugVisualizationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_viewbox);
    }
}

#[derive(Component, Default, PartialEq)]
pub enum DebugVisualizations {
    #[default]
    Hidden,
    Visible,
}

fn draw_viewbox(
    query: Query<(&Handle<VelloVector>, &GlobalTransform, &DebugVisualizations)>,
    query_proj: Query<&OrthographicProjection>,
    vectors: Res<Assets<VelloVector>>,
    mut gizmos: Gizmos,
) {
    let cam_proj = query_proj.single();
    for (vector, transform, _) in query
        .iter()
        .filter(|(_, _, d)| **d == DebugVisualizations::Visible)
    {
        if let Some(vector) = vectors.get(vector) {
            let [(ax, ay), (bx, by), (cx, cy), (dx, dy)] = vector.bb_in_world(transform);

            let points: [([f32; 2], [f32; 2]); 4] = [
                ([ax, ay], [bx, by]),
                ([bx, by], [cx, cy]),
                ([cx, cy], [dx, dy]),
                ([dx, dy], [ax, ay]),
            ];

            for (p_from, p_to) in points {
                let from = Vec2::from(p_from);
                let to = Vec2::from(p_to);

                gizmos.line_2d(from, to, Color::WHITE);
            }

            let origin = Vec2::new((cx + dx) / 2.0, (cy + dy) / 2.0);
            let from = origin + 8.0 * Vec2::splat(1.0) * cam_proj.scale;
            let to = origin + 8.0 * Vec2::splat(-1.0) * cam_proj.scale;

            gizmos.line_2d(from, to, Color::RED);

            let from = origin + 8.0 * Vec2::new(1.0, -1.0) * cam_proj.scale;
            let to = origin + 8.0 * Vec2::new(-1.0, 1.0) * cam_proj.scale;

            gizmos.line_2d(from, to, Color::RED);
        }
    }
}
