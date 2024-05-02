//! Logic for rendering debug visualizations
use crate::text::VelloTextAlignment;
use crate::{CoordinateSpace, VelloAsset, VelloAssetAlignment, VelloFont, VelloText, ZFunction};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

const RED_X_SIZE: f32 = 8.0;

pub struct DebugVisualizationsPlugin;

impl Plugin for DebugVisualizationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (render_asset_debug, render_text_debug));
    }
}

#[derive(Clone, Copy, Component, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub enum DebugVisualizations {
    #[default]
    Hidden,
    Visible,
}

/// A system to render debug visualizations for `VelloAsset`.
fn render_asset_debug(
    query_vectors: Query<
        (
            &Handle<VelloAsset>,
            &VelloAssetAlignment,
            &GlobalTransform,
            &CoordinateSpace,
            &ZFunction,
            &DebugVisualizations,
        ),
        Without<Node>,
    >,
    vectors: Res<Assets<VelloAsset>>,
    query_cam: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<Camera2d>>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, view, projection)) = query_cam.get_single() else {
        return;
    };

    // Show vectors
    for (vector, alignment, gtransform, space, z_fn, _) in query_vectors
        .iter()
        .filter(|(_, _, _, _, _, d)| **d == DebugVisualizations::Visible)
    {
        if let Some(vector) = vectors.get(vector) {
            match space {
                CoordinateSpace::WorldSpace => {
                    // Origin
                    let origin = gtransform.translation().xy();
                    draw_origin(&mut gizmos, projection, origin);
                    // Bounding box
                    let gtransform = &alignment.compute(vector, gtransform);
                    let rect_center = gtransform.translation().xy();
                    let rect = vector.bb_in_world_space(gtransform);
                    draw_bounding_box(&mut gizmos, projection, z_fn, rect_center, rect.size());
                }
                CoordinateSpace::ScreenSpace => {
                    let origin = gtransform.translation().xy();
                    let Some(origin) = camera.viewport_to_world_2d(view, origin) else {
                        continue;
                    };
                    draw_origin(&mut gizmos, projection, origin);
                    // Align BB
                    let gtransform = &alignment.compute(vector, gtransform);
                    let rect_center = gtransform.translation().xy();
                    let Some(rect_center) = camera.viewport_to_world_2d(view, rect_center) else {
                        continue;
                    };
                    let Some(rect) = vector.bb_in_screen_space(gtransform, camera, view) else {
                        continue;
                    };
                    draw_bounding_box(&mut gizmos, projection, z_fn, rect_center, rect.size());
                }
            }
        }
    }
}

/// A system to render debug visualizations for `VelloText`.
fn render_text_debug(
    query_world: Query<
        (
            &Handle<VelloFont>,
            &VelloText,
            &VelloTextAlignment,
            &GlobalTransform,
            &CoordinateSpace,
            &DebugVisualizations,
        ),
        Without<Node>,
    >,
    query_cam: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<Camera2d>>,
    fonts: Res<Assets<VelloFont>>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, view, projection)) = query_cam.get_single() else {
        return;
    };

    // Show world-space vectors
    for (font, text, alignment, gtransform, space, _) in query_world
        .iter()
        .filter(|(_, _, _, _, _, d)| **d == DebugVisualizations::Visible)
    {
        if let Some(font) = fonts.get(font) {
            let rect = text.bb_in_world_space(font, gtransform);
            let mut origin = gtransform.translation().xy();
            match space {
                CoordinateSpace::WorldSpace => {
                    draw_origin(&mut gizmos, projection, origin);
                    let size = rect.size();
                    let (width, height) = size.into();
                    match alignment {
                        VelloTextAlignment::BottomLeft => {}
                        VelloTextAlignment::Bottom => {
                            origin.x += -width / 2.0;
                        }
                        VelloTextAlignment::BottomRight => {
                            origin.x += -width;
                        }
                        VelloTextAlignment::TopLeft => {
                            origin.y += -height;
                        }
                        VelloTextAlignment::Left => {
                            origin.y += -height / 2.0;
                        }
                        VelloTextAlignment::Top => {
                            origin.x += -width / 2.0;
                            origin.y += -height;
                        }
                        VelloTextAlignment::Center => {
                            origin.x += -width / 2.0;
                            origin.y += -height / 2.0;
                        }
                        VelloTextAlignment::TopRight => {
                            origin.x += -width;
                            origin.y += -height;
                        }
                        VelloTextAlignment::Right => {
                            origin.x += -width;
                            origin.y += -height / 2.0;
                        }
                    };
                    let rect_center = origin + rect.size() / 2.0;
                    gizmos.rect_2d(rect_center, 0.0, rect.size(), Color::WHITE);
                }
                CoordinateSpace::ScreenSpace => {
                    let Some(rect) = text.bb_in_screen_space(font, gtransform, camera, view) else {
                        continue;
                    };
                    let Some(mut origin) =
                        camera.viewport_to_world_2d(view, gtransform.translation().xy())
                    else {
                        continue;
                    };
                    draw_origin(&mut gizmos, projection, origin);
                    let size = rect.size();
                    let (width, height) = size.into();
                    match alignment {
                        VelloTextAlignment::BottomLeft => {}
                        VelloTextAlignment::Bottom => {
                            origin.x += -width / 2.0;
                        }
                        VelloTextAlignment::BottomRight => {
                            origin.x += -width;
                        }
                        VelloTextAlignment::TopLeft => {
                            origin.y += height;
                        }
                        VelloTextAlignment::Left => {
                            origin.y += height / 2.0;
                        }
                        VelloTextAlignment::Top => {
                            origin.x += -width / 2.0;
                            origin.y += height;
                        }
                        VelloTextAlignment::Center => {
                            origin.x += -width / 2.0;
                            origin.y += height / 2.0;
                        }
                        VelloTextAlignment::TopRight => {
                            origin.x += -width;
                            origin.y += height;
                        }
                        VelloTextAlignment::Right => {
                            origin.x += -width;
                            origin.y += height / 2.0;
                        }
                    };
                    let rect_center = origin + Vec2::new(rect.width() / 2.0, -rect.height() / 2.0);
                    gizmos.rect_2d(
                        rect_center,
                        0.0,
                        rect.size() * Vec2::new(1.0, 1.0),
                        Color::WHITE,
                    );
                }
            }
        }
    }
}

/// A helper method to draw text gizmos.
fn draw_origin(gizmos: &mut Gizmos, projection: &OrthographicProjection, origin: Vec2) {
    let from = origin + RED_X_SIZE * Vec2::splat(1.0) * projection.scale;
    let to = origin + RED_X_SIZE * Vec2::splat(-1.0) * projection.scale;

    gizmos.line_2d(from, to, Color::RED);

    let from = origin + RED_X_SIZE * Vec2::new(1.0, -1.0) * projection.scale;
    let to = origin + RED_X_SIZE * Vec2::new(-1.0, 1.0) * projection.scale;

    gizmos.line_2d(from, to, Color::RED);
}

/// A helper method to draw the bounding box
fn draw_bounding_box(
    gizmos: &mut Gizmos,
    projection: &OrthographicProjection,
    z_fn: &ZFunction,
    position: Vec2,
    size: Vec2,
) {
    let half_width = size.x / 2.0;
    let half_height = size.y / 2.0;

    // 4 sides
    // Left
    gizmos.line_2d(
        position + Vec2::new(-half_width, -half_height),
        position + Vec2::new(-half_width, half_height),
        Color::WHITE,
    );
    // Top
    gizmos.line_2d(
        position + Vec2::new(-half_width, -half_height),
        position + Vec2::new(half_width, -half_height),
        Color::WHITE,
    );
    // Right
    gizmos.line_2d(
        position + Vec2::new(half_width, -half_height),
        position + Vec2::new(half_width, half_height),
        Color::WHITE,
    );
    // Bottom
    gizmos.line_2d(
        position + Vec2::new(-half_width, half_height),
        position + Vec2::new(half_width, half_height),
        Color::WHITE,
    );

    let from = position + RED_X_SIZE * Vec2::splat(1.0) * projection.scale;
    let to = position + RED_X_SIZE * Vec2::splat(-1.0) * projection.scale;

    gizmos.line_2d(from, to, Color::RED);

    let from = position + RED_X_SIZE * Vec2::new(1.0, -1.0) * projection.scale;
    let to = position + RED_X_SIZE * Vec2::new(-1.0, 1.0) * projection.scale;

    gizmos.line_2d(from, to, Color::RED);

    // TODO: When bevy_gizmos get text, I'd *much rather* just show the Z value with text.
    // See: https://github.com/bevyengine/bevy/issues/9400
    //
    // In addition, everything above should change to:
    // ```rust
    //  gizmos.rect_2d(
    //     position,
    //     0.0,
    //     size,
    //     Color::WHITE,
    // );
    // ```
    const Z_COLOR: Color = Color::GREEN;
    match z_fn {
        ZFunction::TransformX => gizmos.line_2d(
            position + Vec2::new(0.0, -half_height),
            position + Vec2::new(0.0, half_height),
            Z_COLOR,
        ),
        ZFunction::TransformY => gizmos.line_2d(
            position + Vec2::new(-half_width, 0.0),
            position + Vec2::new(half_width, 0.0),
            Z_COLOR,
        ),
        ZFunction::TransformXOffset(offset) => gizmos.line_2d(
            position + Vec2::new(*offset, -half_height),
            position + Vec2::new(*offset, half_height),
            Z_COLOR,
        ),
        ZFunction::TransformYOffset(offset) => gizmos.line_2d(
            position + Vec2::new(-half_width, *offset),
            position + Vec2::new(half_width, *offset),
            Z_COLOR,
        ),
        ZFunction::BbTop => gizmos.line_2d(
            position + Vec2::new(-half_width, half_height),
            position + Vec2::new(half_width, half_height),
            Z_COLOR,
        ),
        ZFunction::BbBottom => gizmos.line_2d(
            position + Vec2::new(-half_width, -half_height),
            position + Vec2::new(half_width, -half_height),
            Z_COLOR,
        ),
        ZFunction::BbLeft => gizmos.line_2d(
            position + Vec2::new(-half_width, -half_height),
            position + Vec2::new(-half_width, half_height),
            Z_COLOR,
        ),
        ZFunction::BbRight => gizmos.line_2d(
            position + Vec2::new(half_width, -half_height),
            position + Vec2::new(half_width, half_height),
            Z_COLOR,
        ),
        ZFunction::TransformZ
        | ZFunction::TransformZOffset(_)
        | ZFunction::Computed(_)
        | ZFunction::Value(_) => {
            // No way to display this
        }
    }
}
