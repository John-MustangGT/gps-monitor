// src/display/gui/panels.rs v1
//! Main GPS data panel rendering

use crate::gps::GpsData;
use chrono::TimeZone;
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
    // Large GPS Time Display
    ui.vertical_centered(|ui| {
        ui.add_space(5.0);

        let time_text = match data.timestamp {
            Some(ts) => {
                let local_time = ts.with_timezone(&chrono::Local);
                format!("{}", local_time.format("%H:%M:%S"))
            }
            None => "No GPS Time".to_string(),
        };

        // Large time display with custom font size
        ui.heading(egui::RichText::new("🕐 GPS Time").size(16.0));
        ui.label(
            egui::RichText::new(time_text)
                .size(48.0)
                .strong()
                .monospace()
                .color(egui::Color32::from_rgb(100, 200, 255))
        );

        // Show date below time
        if let Some(ts) = data.timestamp {
            let local_time = ts.with_timezone(&chrono::Local);
            ui.label(
                egui::RichText::new(format!("{}", local_time.format("%Y-%m-%d")))
                    .size(14.0)
                    .color(egui::Color32::GRAY)
            );
        }

        ui.add_space(5.0);
    });

    ui.separator();
    ui.add_space(10.0);

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
