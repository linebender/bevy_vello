use bevy::{
    asset::AssetMetaCheck,
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        ButtonState,
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_vello::{debug::DebugVisualizations, Origin, VelloPlugin, VelloVector};

mod editor;
pub mod util;

#[derive(Component)]
struct Selected;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin { ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("#c".to_string()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(VelloPlugin)
        .add_plugins(editor::EditorPlugin)
        .add_systems(Startup, setup_vector_graphics)
        .add_systems(
            Update,
            (
                camera_system,
                update_vector_selection,
                update_debug_selected,
            ),
        )
        .run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 4.0,
            ..default()
        },
        ..default()
    });
    util::spawn_vector(asset_server.load("../assets/squid.json"), &mut commands);
}

#[derive(Component)]
pub struct CameraFollow;

fn camera_system(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut query_cam: Query<
        (
            Entity,
            &mut OrthographicProjection,
            &mut Transform,
            Option<&CameraFollow>,
        ),
        (With<Camera>, Without<Handle<VelloVector>>),
    >,
    mut scroll_events: EventReader<MouseWheel>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut mouse_motion_evr: EventReader<MouseMotion>,
    mut commands: Commands,
) {
    let Ok((entity, mut projection, mut transform, follow_camera)) = query_cam.get_single_mut()
    else {
        return;
    };

    let Ok(window) = query_windows.get_single() else {
        return;
    };

    if let Some(scroll) = scroll_events.read().next() {
        // Get the cursor position relative to the center so the camera zooms in/out from cursor
        let origin = Vec2::new(window.width() / 2.0, window.height() / 2.0);
        let mut cursor_pos = window.cursor_position().unwrap_or(origin);
        cursor_pos.y = window.height() - cursor_pos.y;
        let cursor_offset = cursor_pos - origin;

        const ZOOM_SCALE: f32 = 0.5;
        let zoom_factor = -scroll.y * ZOOM_SCALE * f32::sqrt(projection.scale);
        let translation = (cursor_offset * (projection.scale + zoom_factor))
            - (cursor_offset * (projection.scale));

        let new_scale = (projection.scale + zoom_factor).clamp(0.5, 60.0);
        if new_scale != projection.scale {
            transform.translation -= translation.extend(0.0);
            projection.scale = new_scale;
        }
    }

    // Update the camera position when dragging the mouse
    if follow_camera.is_some() {
        for event in mouse_motion_evr.read() {
            let delta = Vec2::new(event.delta.x, -event.delta.y);
            transform.translation -= delta.extend(0.0) * projection.scale;
        }
    }
    if let Some(event) = mousebtn_evr.read().next() {
        if event.button == MouseButton::Right {
            match event.state {
                ButtonState::Pressed => {
                    commands.entity(entity).insert(CameraFollow);
                    mouse_motion_evr.clear();
                }
                ButtonState::Released => {
                    commands.entity(entity).remove::<CameraFollow>();
                }
            }
        }
    }
}

fn update_vector_selection(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    query_cam: Query<(&Camera, &GlobalTransform), (With<Camera>, Without<Handle<VelloVector>>)>,
    query_clickables: Query<(Entity, &Handle<VelloVector>, &GlobalTransform, &Origin)>,
    query_selected: Query<Entity, With<Selected>>,
    vector_assets: Res<Assets<VelloVector>>,
    side_panel: Res<editor::SidePanel>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut commands: Commands,
) {
    let Ok((camera, camera_transform)) = query_cam.get_single() else {
        return;
    };

    let Ok(window) = query_windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    else {
        return;
    };

    // Make sure we don't consider mouse events when the mouse is over the side panel.
    if window
        .cursor_position()
        .map(|cursor| cursor.x > (window.width() - side_panel.occupied_width()))
        .unwrap_or(true)
    {
        mousebtn_evr.clear();
        return;
    }

    for event in mousebtn_evr.read() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            for entity in query_selected.iter() {
                commands.entity(entity).remove::<Selected>();
            }

            // Sort the entities by their Z position so that the top-most entity is selected first
            let mut clickables = query_clickables.iter().collect::<Vec<_>>();
            clickables.sort_by(|(_, _, a, _), (_, _, b, _)| {
                let a = a.translation().z;
                let b = b.translation().z;
                a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
            });

            for (entity, vector_handle, transform, origin) in clickables {
                let Some(vector) = vector_assets.get(vector_handle) else {
                    continue;
                };

                let [min, x_axis, max, y_axis] = vector.bb_in_world(transform, origin);

                fn calc_tri_area(a: Vec2, b: Vec2, c: Vec2) -> f32 {
                    let a = a - c;
                    let b = b - c;
                    (a.x * b.y - a.y * b.x).abs() / 2.0
                }
                let area_a = calc_tri_area(min, x_axis, cursor_pos);
                let area_d = calc_tri_area(min, y_axis, cursor_pos);
                let area_b = calc_tri_area(x_axis, max, cursor_pos);
                let area_c = calc_tri_area(y_axis, max, cursor_pos);
                let cursor_area = area_a + area_b + area_c + area_d;
                let bb_area = calc_tri_area(min, max, x_axis) + calc_tri_area(min, max, y_axis);

                // Cursor area should be equal to bounding box area if the cursor is inside the bounding box
                let in_bounds = cursor_area <= bb_area + 0.1 && cursor_area >= bb_area - 0.1;

                if in_bounds {
                    commands.entity(entity).insert(Selected);
                    return;
                }
            }
        }
    }
}

// Updates any entities with the Selected component to show debug visualizations so they look
// selected.
fn update_debug_selected(
    query_selected: Query<(Entity, Option<&Selected>), With<Handle<VelloVector>>>,
    mut commands: Commands,
) {
    for (entity, selected) in query_selected.iter() {
        if selected.is_none() {
            commands.entity(entity).remove::<DebugVisualizations>();
        } else {
            commands
                .entity(entity)
                .try_insert(DebugVisualizations::Visible);
        }
    }
}
