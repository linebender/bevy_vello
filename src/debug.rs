use crate::VelloAsset;
use bevy::{math::Vec3Swizzles, prelude::*};

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
    query_world: Query<
        (&Handle<VelloAsset>, &GlobalTransform, &DebugVisualizations),
        Without<Node>,
    >,
    query_ui: Query<(&Handle<VelloAsset>, &GlobalTransform, &DebugVisualizations), With<Node>>,
    vectors: Res<Assets<VelloAsset>>,
    query_cam: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<Camera2d>>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, view, projection)) = query_cam.get_single() else {
        return;
    };

    const RED_X_SIZE: f32 = 8.0;

    // Show world-space vectors
    for (vector, transform, _) in query_world
        .iter()
        .filter(|(_, _, d)| **d == DebugVisualizations::Visible)
    {
        if let Some(vector) = vectors.get(vector) {
            let [min, x_axis, max, y_axis] = vector.bb_in_world_space(transform);

            gizmos.line_2d(min, x_axis, Color::WHITE);
            gizmos.line_2d(min, y_axis, Color::WHITE);
            gizmos.line_2d(x_axis, max, Color::WHITE);
            gizmos.line_2d(y_axis, max, Color::WHITE);

            let red_x_origin = transform.translation().xy();
            let from = red_x_origin + RED_X_SIZE * Vec2::splat(1.0) * projection.scale;
            let to = red_x_origin + RED_X_SIZE * Vec2::splat(-1.0) * projection.scale;

            gizmos.line_2d(from, to, Color::RED);

            let from = red_x_origin + RED_X_SIZE * Vec2::new(1.0, -1.0) * projection.scale;
            let to = red_x_origin + RED_X_SIZE * Vec2::new(-1.0, 1.0) * projection.scale;

            gizmos.line_2d(from, to, Color::RED);
        }
    }

    // Show screen-space vectors
    for (vector, transform, _) in query_ui
        .iter()
        .filter(|(_, _, d)| **d == DebugVisualizations::Visible)
    {
        if let Some(vector) = vectors.get(vector) {
            let &[Some(min), Some(x_axis), Some(max), Some(y_axis)] = vector
                .bb_in_screen_space(transform)
                .iter()
                .map(|&v| camera.viewport_to_world_2d(view, v))
                .collect::<Vec<Option<Vec2>>>()
                .as_slice()
            else {
                continue;
            };

            gizmos.line_2d(min, x_axis, Color::WHITE);
            gizmos.line_2d(min, y_axis, Color::WHITE);
            gizmos.line_2d(x_axis, max, Color::WHITE);
            gizmos.line_2d(y_axis, max, Color::WHITE);

            let red_x_origin = Vec2::new((y_axis.x + max.x) / 2.0, (y_axis.y + min.y) / 2.0);
            let from = red_x_origin + RED_X_SIZE * Vec2::splat(1.0) * projection.scale;
            let to = red_x_origin + RED_X_SIZE * Vec2::splat(-1.0) * projection.scale;

            gizmos.line_2d(from, to, Color::RED);

            let from = red_x_origin + RED_X_SIZE * Vec2::new(1.0, -1.0) * projection.scale;
            let to = red_x_origin + RED_X_SIZE * Vec2::new(-1.0, 1.0) * projection.scale;

            gizmos.line_2d(from, to, Color::RED);
        }
    }
}
