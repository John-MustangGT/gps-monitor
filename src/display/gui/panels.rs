// src/display/gui/panels.rs v1
//! Main GPS data panel rendering

use crate::gps::GpsData;
use eframe::egui;

fn format_coordinate(coord: Option<f64>) -> String {
    match coord {
        Some(val) => format!("{:.6}°", val),
        None => "No fix".to_string(),
    }
}

fn format_value<T: std::fmt::Display>(value: Option<T>, unit: &str) -> String {
    match value {
        Some(val) => format!("{} {}", val, unit),
        None => "Unknown".to_string(),
    }
}

pub fn render_main_data_panel(ui: &mut egui::Ui, data: &GpsData) {
    ui.strong("📍 Position & Movement");
    ui.separator();

    // Position section
    egui::Grid::new("position_grid")
        .num_columns(2)
        .spacing([10.0, 8.0])
        .show(ui, |ui| {
            ui.label("Latitude:");
            ui.monospace(format_coordinate(data.latitude));
            ui.end_row();

            ui.label("Longitude:");
            ui.monospace(format_coordinate(data.longitude));
            ui.end_row();

            ui.label("Altitude:");
            ui.monospace(format_value(data.altitude, "m"));
            ui.end_row();

            if let Some(accuracy) = data.accuracy {
                ui.label("Accuracy:");
                ui.monospace(format!("{:.1} m", accuracy));
                ui.end_row();
            }
        });

    ui.add_space(10.0);

    // Movement section
    egui::Grid::new("movement_grid")
        .num_columns(2)
        .spacing([10.0, 8.0])
        .show(ui, |ui| {
            ui.label("Speed:");
            ui.monospace(format_value(data.speed, "km/h"));
            ui.end_row();

            ui.label("Course:");
            ui.monospace(format_value(data.course, "°"));
            ui.end_row();
        });

    ui.add_space(10.0);

    // Signal Quality section (if GPS data available)
    if data.satellites.is_some() || data.hdop.is_some() || data.fix_quality.is_some() {
        ui.strong("📡 Signal Quality");
        ui.separator();
        
        egui::Grid::new("quality_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                if let Some(sats) = data.satellites {
                    ui.label("Satellites:");
                    ui.monospace(format!("{}", sats));
                    ui.end_row();
                }

                if let Some(hdop) = data.hdop {
                    ui.label("HDOP:");
                    ui.monospace(format!("{:.1}", hdop));
                    ui.end_row();
                }

                ui.label("Fix Type:");
                ui.monospace(data.get_fix_description());
                ui.end_row();
            });
    }
}
