// src/display/gui/waypoint_dialog.rs v4
//! Waypoint recording dialog UI

use crate::{gps::GpsData, waypoint::{Waypoint, WaypointExporter, WaypointFormat}};
use eframe::egui;
use std::path::PathBuf;

pub struct WaypointDialog {
    pub open: bool,
    waypoint_name: String,
    waypoint_description: String,
    exporter: WaypointExporter,
    selected_format: WaypointFormat,
    export_path: String,
    status_message: Option<String>,
}

impl WaypointDialog {
    pub fn new() -> Self {
        Self {
            open: false,
            waypoint_name: String::new(),
            waypoint_description: String::new(),
            exporter: WaypointExporter::new(),
            selected_format: WaypointFormat::GPX,
            export_path: String::new(),
            status_message: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, gps_data: &GpsData) {
        if !self.open {
            return;
        }

        egui::Window::new("📍 Waypoint Manager")
            .collapsible(false)
            .resizable(true)
            .default_width(450.0)
            .show(ctx, |ui| {
                // Add close button in header
                ui.horizontal(|ui| {
                    ui.heading("Record Waypoints");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖").clicked() {
                            self.open = false;
                        }
                    });
                });
                ui.separator();

                // Current position info
                ui.group(|ui| {
                    ui.label("Current Position:");
                    egui::Grid::new("current_pos_grid")
                        .num_columns(2)
                        .spacing([10.0, 5.0])
                        .show(ui, |ui| {
                            ui.label("Latitude:");
                            ui.monospace(GpsData::format_coordinate(gps_data.latitude));
                            ui.end_row();

                            ui.label("Longitude:");
                            ui.monospace(GpsData::format_coordinate(gps_data.longitude));
                            ui.end_row();

                            if let Some(alt) = gps_data.altitude {
                                ui.label("Altitude:");
                                ui.monospace(format!("{:.1} m", alt));
                                ui.end_row();
                            }
                        });
                });

                ui.add_space(10.0);

                // Waypoint input
                ui.group(|ui| {
                    ui.label("New Waypoint:");
                    
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.waypoint_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.text_edit_singleline(&mut self.waypoint_description);
                    });

                    ui.add_space(5.0);

                    let can_save = gps_data.has_fix() && !self.waypoint_name.is_empty();
                    
                    ui.horizontal(|ui| {
                        if ui.add_enabled(can_save, egui::Button::new("💾 Save Waypoint")).clicked() {
                            self.save_waypoint(gps_data);
                        }

                        if !can_save {
                            if !gps_data.has_fix() {
                                ui.colored_label(egui::Color32::YELLOW, "⚠ No GPS fix");
                            } else if self.waypoint_name.is_empty() {
                                ui.colored_label(egui::Color32::YELLOW, "⚠ Name required");
                            }
                        }
                    });
                });

                ui.add_space(10.0);
                ui.separator();

                // Saved waypoints list
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(format!("Saved Waypoints ({})", self.exporter.waypoint_count()));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑 Clear All").clicked() {
                                self.exporter.clear();
                                self.status_message = Some("All waypoints cleared".to_string());
                            }
                        });
                    });

                    ui.separator();

                    if self.exporter.waypoint_count() == 0 {
                        ui.weak("No waypoints saved yet");
                    } else {
                        egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                            egui::Grid::new("waypoint_list")
                                .num_columns(3)
                                .spacing([10.0, 5.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.strong("Name");
                                    ui.strong("Position");
                                    ui.strong("Time");
                                    ui.end_row();

                                    for wp in self.exporter.get_waypoints() {
                                        ui.label(&wp.name);
                                        ui.monospace(format!("{:.6}, {:.6}", wp.latitude, wp.longitude));
                                        ui.monospace(wp.timestamp.format("%H:%M:%S").to_string());
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                });

                ui.add_space(10.0);
                ui.separator();

                // Export section
                ui.group(|ui| {
                    ui.strong("Export Waypoints");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_id_source("format_selector")
                            .selected_text(self.selected_format.display_name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.selected_format, WaypointFormat::GPX, WaypointFormat::GPX.display_name());
                                ui.selectable_value(&mut self.selected_format, WaypointFormat::GeoJSON, WaypointFormat::GeoJSON.display_name());
                                ui.selectable_value(&mut self.selected_format, WaypointFormat::KML, WaypointFormat::KML.display_name());
                                ui.selectable_value(&mut self.selected_format, WaypointFormat::CSV, WaypointFormat::CSV.display_name());
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Filename:");
                        ui.text_edit_singleline(&mut self.export_path);
                        ui.label(format!(".{}", self.selected_format.extension()));
                    });

                    ui.add_space(5.0);

                    let can_export = self.exporter.waypoint_count() > 0 && !self.export_path.is_empty();

                    if ui.add_enabled(can_export, egui::Button::new("💾 Export to File")).clicked() {
                        self.export_waypoints();
                    }

                    if !can_export && self.exporter.waypoint_count() == 0 {
                        ui.colored_label(egui::Color32::YELLOW, "⚠ No waypoints to export");
                    }
                });

                // Status message
                if let Some(ref msg) = self.status_message {
                    ui.add_space(5.0);
                    ui.separator();
                    ui.colored_label(egui::Color32::GREEN, msg);
                }

                ui.add_space(5.0);
                ui.separator();

                // Close button at bottom
                if ui.button("❌ Close").clicked() {
                    self.open = false;
                }
            });
    }

    fn save_waypoint(&mut self, gps_data: &GpsData) {
        let desc = if self.waypoint_description.is_empty() {
            None
        } else {
            Some(self.waypoint_description.clone())
        };

        if let Some(waypoint) = Waypoint::from_gps_data(
            gps_data,
            self.waypoint_name.clone(),
            desc,
        ) {
            self.exporter.add_waypoint(waypoint);
            self.status_message = Some(format!("Waypoint '{}' saved!", self.waypoint_name));
            
            // Clear input fields
            self.waypoint_name.clear();
            self.waypoint_description.clear();
        } else {
            self.status_message = Some("Error: No valid GPS position".to_string());
        }
    }

    fn export_waypoints(&mut self) {
        let mut path = PathBuf::from(&self.export_path);
        
        // Add extension if not present
        if path.extension().is_none() {
            path.set_extension(self.selected_format.extension());
        }

        match self.exporter.export_to_file(&path, self.selected_format) {
            Ok(_) => {
                self.status_message = Some(format!(
                    "✓ Exported {} waypoints to {}",
                    self.exporter.waypoint_count(),
                    path.display()
                ));
            }
            Err(e) => {
                self.status_message = Some(format!("✗ Export failed: {}", e));
            }
        }
    }
}

impl Default for WaypointDialog {
    fn default() -> Self {
        Self::new()
    }
}
