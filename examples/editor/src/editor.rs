use bevy::prelude::*;
use bevy_egui::{
    egui::{Align, DragValue, Layout},
    EguiContexts, EguiPlugin,
};

use crate::{util, Selected};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<SidePanel>()
            .add_systems(Update, ui_system);
    }
}

#[derive(Resource, Default)]
pub struct SidePanel {
    occupied_width: f32,
}

impl SidePanel {
    pub fn occupied_width(&self) -> f32 {
        self.occupied_width
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut selected: Query<(Entity, &mut Transform), With<Selected>>,
    mut side_panel: ResMut<SidePanel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let ctx = contexts.ctx_mut();

    side_panel.occupied_width = bevy_egui::egui::SidePanel::right("side_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.with_layout(
                Layout::top_down(Align::LEFT).with_main_justify(true),
                |ui| {
                    ui.group(|ui| {
                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            ui.heading("Properties");
                            ui.separator();

                            ui.push_id("properties_area", |ui| {
                                bevy_egui::egui::ScrollArea::new([true, true]).show(ui, |ui| {
                                    let selected_vector = selected.get_single_mut();
                                    if let Ok((entity, mut transform)) = selected_vector {
                                        ui.label("Position:");
                                        ui.horizontal(|group| {
                                            group.label("X:");
                                            group.add(DragValue::new(&mut transform.translation.x));
                                            group.label("Y:");
                                            group.add(DragValue::new(&mut transform.translation.y));
                                            group.label("Z:");
                                            group.add(DragValue::new(&mut transform.translation.z));
                                        });
                                        ui.label("Rotation:");
                                        ui.horizontal(|group| {
                                            group.label("Deg:");
                                            let mut z_rotation = -transform
                                                .rotation
                                                .to_euler(EulerRot::XYZ)
                                                .2
                                                .to_degrees();
                                            group.add(DragValue::new(&mut z_rotation));
                                            transform.rotation =
                                                Quat::from_rotation_z((-z_rotation).to_radians());
                                        });
                                        ui.label("Scale:");
                                        ui.horizontal(|group| {
                                            group.label("X:");
                                            group.add(
                                                DragValue::new(&mut transform.scale.x)
                                                    .speed(0.02)
                                                    .clamp_range(0.0..=f32::INFINITY),
                                            );
                                            group.label("Y:");
                                            group.add(
                                                DragValue::new(&mut transform.scale.y)
                                                    .speed(0.02)
                                                    .clamp_range(0.0..=f32::INFINITY),
                                            );
                                        });
                                        if ui.button("Delete").clicked() {
                                            commands.entity(entity).despawn_recursive();
                                        }
                                    } else {
                                        ui.label("Nothing selected");
                                    }
                                });
                            });
                        });
                    });
                    ui.group(|ui| {
                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            ui.heading("Editor");
                            ui.separator();
                            ui.push_id("editor_area", |ui| {
                                bevy_egui::egui::ScrollArea::new([true, true]).show(ui, |ui| {
                                    if ui.button("Add Squid").clicked() {
                                        util::spawn_vector(
                                            asset_server.load("../assets/squid.json"),
                                            &mut commands,
                                        );
                                    }
                                    #[cfg(not(target_arch = "wasm32"))]
                                    if ui.button("Add Custom File...").clicked() {
                                        if let Some(file_path) = rfd::FileDialog::new().pick_file()
                                        {
                                            util::spawn_vector(
                                                asset_server.load(file_path),
                                                &mut commands,
                                            );
                                        }
                                    }
                                });
                            });
                        });
                    });
                },
            );
        })
        .response
        .rect
        .width();
}
