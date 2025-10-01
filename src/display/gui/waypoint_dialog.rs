// src/display/gui/waypoint_dialog.rs v5
//! Waypoint recording and track recording dialog UI

use crate::{gps::GpsData, waypoint::{Waypoint, WaypointExporter, WaypointFormat}};
use super::track_recorder::TrackRecorder;
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
    
    // Track recording
    track_recorder: TrackRecorder,
    track_name_input: String,
    show_track_settings: bool,
    min_distance_str: String,
    min_time_str: String,
}

impl WaypointDialog {
    pub fn new() -> Self {
        let track_recorder = TrackRecorder::new();
        Self {
            open: false,
            waypoint_name: String::new(),
            waypoint_description: String::new(),
            exporter: WaypointExporter::new(),
            selected_format: WaypointFormat::GPX,
            export_path: String::new(),
            status_message: None,
            track_name_input: String::new(),
            show_track_settings: false,
            min_distance_str: track_recorder.get_min_distance().to_string(),
            min_time_str: track_recorder.get_min_time_seconds().to_string(),
            track_recorder,
        }
    }

    pub fn update_from_gps(&mut self, gps_data: &GpsData) {
        self.track_recorder.update(gps_data);
    }

    pub fn show(&mut self, ctx: &egui::Context, gps_data: &GpsData) {
        if !self.open {
            return;
        }

        egui::Window::new("📍 Waypoint & Track Manager")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .show(ctx, |ui| {
                // Header with tabs
                ui.horizontal(|ui| {
                    ui.heading("GPS Recording");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖").clicked() {
                            self.open = false;
                        }
                    });
                });
                ui.separator();

                // Tab selection
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.show_track_settings, false, "📍 Waypoints");
                    ui.selectable_value(&mut self.show_track_settings, true, "📊 Track Recording");
                });
                ui.separator();

                if !self.show_track_settings {
                    self.render_waypoint_tab(ui, gps_data);
                } else {
                    self.render_track_tab(ui, gps_data);
                }

                // Status message (shared)
                if let Some(ref msg) = self.status_message {
                    ui.add_space(5.0);
                    ui.separator();
                    ui.colored_label(egui::Color32::GREEN, msg);
                }

                ui.add_space(5.0);
                ui.separator();

                // Summary and export section
                self.render_export_section(ui);
            });
    }

    fn render_waypoint_tab(&mut self, ui: &mut egui::Ui, gps_data: &GpsData) {
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

        // Saved waypoints list
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("Saved Waypoints ({})", self.exporter.waypoint_count()));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Clear").clicked() {
                        self.exporter.clear_waypoints();
                        self.status_message = Some("Waypoints cleared".to_string());
                    }
                });
            });

            ui.separator();

            if self.exporter.waypoint_count() == 0 {
                ui.weak("No waypoints saved yet");
            } else {
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
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
    }

    fn render_track_tab(&mut self, ui: &mut egui::Ui, gps_data: &GpsData) {
        // Recording controls
        ui.group(|ui| {
            ui.strong("Track Recording");
            ui.separator();

            if !self.track_recorder.is_recording() {
                ui.horizontal(|ui| {
                    ui.label("Track Name:");
                    ui.text_edit_singleline(&mut self.track_name_input);
                });

                ui.add_space(5.0);

                let can_start = gps_data.has_fix();
                ui.horizontal(|ui| {
                    if ui.add_enabled(can_start, egui::Button::new("🔴 Start Recording")).clicked() {
                        self.track_recorder.start_recording(self.track_name_input.clone());
                        self.status_message = Some(format!("Recording started: {}", self.track_recorder.get_track_name()));
                    }

                    if !can_start {
                        ui.colored_label(egui::Color32::YELLOW, "⚠ No GPS fix");
                    }
                });
            } else {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), 
                    format!("🔴 Recording: {}", self.track_recorder.get_track_name()));
                
                ui.add_space(5.0);

                // Show stats
                if let Some(stats) = self.track_recorder.get_track_stats() {
                    egui::Grid::new("track_stats")
                        .num_columns(2)
                        .spacing([10.0, 5.0])
                        .show(ui, |ui| {
                            ui.label("Points:");
                            ui.monospace(format!("{}", stats.points));
                            ui.end_row();

                            ui.label("Distance:");
                            ui.monospace(format!("{:.2} km", stats.distance_km));
                            ui.end_row();

                            ui.label("Duration:");
                            ui.monospace(stats.format_duration());
                            ui.end_row();

                            if let Some(avg_speed) = stats.avg_speed {
                                ui.label("Avg Speed:");
                                ui.monospace(format!("{:.1} km/h", avg_speed));
                                ui.end_row();
                            }
                        });
                }

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("⏹ Stop & Save").clicked() {
                        if let Some(track) = self.track_recorder.stop_recording() {
                            self.exporter.add_track(track);
                            self.status_message = Some("Track saved!".to_string());
                            self.track_name_input.clear();
                        }
                    }

                    if ui.button("⏸ Pause").clicked() {
                        self.track_recorder.pause_recording();
                        self.status_message = Some("Track paused (new segment on resume)".to_string());
                    }

                    if ui.button("❌ Discard").clicked() {
                        self.track_recorder.stop_recording();
                        self.status_message = Some("Track discarded".to_string());
                        self.track_name_input.clear();
                    }
                });
            }
        });

        ui.add_space(10.0);

        // Recording settings
        ui.group(|ui| {
            ui.strong("Recording Settings");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Min Distance:");
                if ui.text_edit_singleline(&mut self.min_distance_str).changed() {
                    if let Ok(val) = self.min_distance_str.parse::<f64>() {
                        self.track_recorder.set_min_distance(val);
                    }
                }
                ui.label("meters");
            });

            ui.horizontal(|ui| {
                ui.label("Min Time:");
                if ui.text_edit_singleline(&mut self.min_time_str).changed() {
                    if let Ok(val) = self.min_time_str.parse::<u64>() {
                        self.track_recorder.set_min_time(val);
                    }
                }
                ui.label("seconds");
            });

            ui.add_space(3.0);
            ui.small("Points recorded only when both thresholds exceeded");
        });

        ui.add_space(10.0);

        // Saved tracks list
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("Saved Tracks ({})", self.exporter.track_count()));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Clear").clicked() {
                        self.exporter.clear_tracks();
                        self.status_message = Some("Tracks cleared".to_string());
                    }
                });
            });

            ui.separator();

            if self.exporter.track_count() == 0 {
                ui.weak("No tracks saved yet");
            } else {
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    egui::Grid::new("track_list")
                        .num_columns(3)
                        .spacing([10.0, 5.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("Name");
                            ui.strong("Points");
                            ui.strong("Distance");
                            ui.end_row();

                            for track in self.exporter.get_tracks() {
                                ui.label(&track.name);
                                ui.monospace(format!("{}", track.total_points()));
                                ui.monospace(format!("{:.2} km", track.total_distance() / 1000.0));
                                ui.end_row();
                            }
                        });
                });
            }
        });
    }

    fn render_export_section(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            let total_items = self.exporter.waypoint_count() + self.exporter.track_count();
            ui.horizontal(|ui| {
                ui.strong(format!("Export Data ({} waypoints, {} tracks)", 
                    self.exporter.waypoint_count(), 
                    self.exporter.track_count()));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Clear All").clicked() {
                        self.exporter.clear();
                        self.status_message = Some("All data cleared".to_string());
                    }
                });
            });

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

            let can_export = total_items > 0 && !self.export_path.is_empty();

            if ui.add_enabled(can_export, egui::Button::new("💾 Export to File")).clicked() {
                self.export_data();
            }

            if !can_export && total_items == 0 {
                ui.colored_label(egui::Color32::YELLOW, "⚠ No data to export");
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

    fn export_data(&mut self) {
        let mut path = PathBuf::from(&self.export_path);
        
        // Add extension if not present
        if path.extension().is_none() {
            path.set_extension(self.selected_format.extension());
        }

        match self.exporter.export_to_file(&path, self.selected_format) {
            Ok(_) => {
                self.status_message = Some(format!(
                    "✓ Exported {} waypoints and {} tracks to {}",
                    self.exporter.waypoint_count(),
                    self.exporter.track_count(),
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
