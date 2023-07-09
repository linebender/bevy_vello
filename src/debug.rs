use bevy::prelude::*;

use crate::VelloVector;

pub struct DebugVisualizationsPlugin;

impl Plugin for DebugVisualizationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_viewbox);
    }
}

#[derive(Clone, Copy, Component, Default, PartialEq, Reflect)]
#[reflect(Component)]
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
            let [min, x_axis, max, y_axis] = vector.bb_in_world(transform);

            gizmos.line_2d(min, x_axis, Color::WHITE);
            gizmos.line_2d(min, y_axis, Color::WHITE);
            gizmos.line_2d(x_axis, max, Color::WHITE);
            gizmos.line_2d(y_axis, max, Color::WHITE);

            let origin = Vec2::new((y_axis.x + max.x) / 2.0, (y_axis.y + max.y) / 2.0);
            let from = origin + 8.0 * Vec2::splat(1.0) * cam_proj.scale;
            let to = origin + 8.0 * Vec2::splat(-1.0) * cam_proj.scale;

            gizmos.line_2d(from, to, Color::RED);

            let from = origin + 8.0 * Vec2::new(1.0, -1.0) * cam_proj.scale;
            let to = origin + 8.0 * Vec2::new(-1.0, 1.0) * cam_proj.scale;

            gizmos.line_2d(from, to, Color::RED);
        }
    }
}
