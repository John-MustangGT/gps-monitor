// src/display/gui/skyplot.rs v1
//! Sky plot rendering - polar coordinate satellite visualization

use crate::gps::GpsData;
use eframe::egui;

pub fn render_sky_plot(ui: &mut egui::Ui, data: &GpsData) {
    ui.strong("🌌 Sky Plot");
    ui.separator();

    if data.satellites_info.is_empty() {
        ui.weak("No satellite position data");
        return;
    }

    // Calculate responsive plot size
    let available_size = ui.available_size();
    let max_plot_size = available_size.x.min(available_size.y - 60.0);
    let plot_size = max_plot_size.max(150.0).min(350.0);
    let radius = plot_size / 2.0 - 20.0;

    // Allocate space for the plot
    let (rect, _response) = ui.allocate_exact_size(
        [plot_size, plot_size].into(),
        egui::Sense::hover()
    );

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        
        draw_background(painter, rect.center(), radius);
        draw_cardinal_directions(painter, rect.center(), radius);
        draw_satellites(painter, rect.center(), radius, plot_size, data);
        draw_elevation_labels(painter, rect.center(), radius, plot_size);
    }

    // Legend
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.small("Legend:");
        ui.colored_label(egui::Color32::from_rgb(0, 150, 255), "● GPS");
        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "● GLO");
        ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "● GAL");
        ui.colored_label(egui::Color32::from_rgb(255, 255, 100), "● BDS");
    });
}

fn draw_background(painter: &egui::Painter, center: egui::Pos2, radius: f32) {
    // Horizon circle
    painter.circle_stroke(
        center,
        radius,
        egui::Stroke::new(2.0, egui::Color32::GRAY)
    );

    // 60° elevation circle
    painter.circle_stroke(
        center,
        radius * 2.0 / 3.0,
        egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
    );

    // 30° elevation circle
    painter.circle_stroke(
        center,
        radius / 3.0,
        egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
    );
}

fn draw_cardinal_directions(painter: &egui::Painter, center: egui::Pos2, radius: f32) {
    let directions: [(f32, &str); 4] = [
        (0.0, "N"),
        (90.0, "E"),
        (180.0, "S"),
        (270.0, "W"),
    ];

    for (angle_deg, label) in directions {
        let angle_rad = angle_deg.to_radians();
        let end_pos = center + egui::vec2(
            angle_rad.sin() * radius,
            -angle_rad.cos() * radius
        );
        
        // Direction line
        painter.line_segment(
            [center, end_pos],
            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
        );

        // Direction label
        let label_pos = center + egui::vec2(
            angle_rad.sin() * (radius + 10.0),
            -angle_rad.cos() * (radius + 10.0)
        );
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::default(),
            egui::Color32::WHITE
        );
    }
}

fn draw_satellites(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    plot_size: f32,
    data: &GpsData
) {
    for sat in &data.satellites_info {
        if let (Some(elevation), Some(azimuth)) = (sat.elevation, sat.azimuth) {
            // Convert polar to screen coordinates
            let elev_normalized = (90.0 - elevation) / 90.0;
            let sat_radius = radius * elev_normalized;
            
            let azimuth_rad = azimuth.to_radians();
            let sat_pos = center + egui::vec2(
                azimuth_rad.sin() * sat_radius,
                -azimuth_rad.cos() * sat_radius
            );

            // Determine color and size based on constellation and usage
            let (sat_color, sat_size) = get_satellite_style(sat, plot_size);

            // Draw satellite dot
            painter.circle_filled(sat_pos, sat_size, sat_color);

            // Draw PRN label
            let text_pos = sat_pos + egui::vec2(sat_size + 2.0, 0.0);
            let font_size = (plot_size / 25.0).max(8.0).min(12.0);
            painter.text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                sat.prn.to_string(),
                egui::FontId::monospace(font_size),
                egui::Color32::WHITE
            );

            // Draw signal strength ring for used satellites
            if sat.used {
                if let Some(snr) = sat.snr {
                    let ring_color = get_snr_color(snr);
                    painter.circle_stroke(
                        sat_pos,
                        sat_size + 2.0,
                        egui::Stroke::new(1.5, ring_color)
                    );
                }
            }
        }
    }
}

fn get_satellite_style(sat: &crate::gps::data::SatelliteInfo, plot_size: f32) -> (egui::Color32, f32) {
    if sat.used {
        let color = match sat.constellation.as_str() {
            "GPS" => egui::Color32::from_rgb(0, 150, 255),
            "GLONASS" => egui::Color32::from_rgb(255, 100, 100),
            "GALILEO" => egui::Color32::from_rgb(100, 255, 100),
            "BEIDOU" => egui::Color32::from_rgb(255, 255, 100),
            "QZSS" => egui::Color32::from_rgb(255, 150, 0),
            _ => egui::Color32::WHITE,
        };
        let size = (plot_size / 30.0).max(4.0).min(10.0);
        (color, size)
    } else {
        let size = (plot_size / 50.0).max(3.0).min(6.0);
        (egui::Color32::GRAY, size)
    }
}

fn get_snr_color(snr: f32) -> egui::Color32 {
    match snr {
        s if s >= 40.0 => egui::Color32::GREEN,
        s if s >= 35.0 => egui::Color32::YELLOW,
        s if s >= 25.0 => egui::Color32::from_rgb(255, 165, 0),
        _ => egui::Color32::RED,
    }
}

fn draw_elevation_labels(painter: &egui::Painter, center: egui::Pos2, radius: f32, plot_size: f32) {
    let label_font_size = (plot_size / 30.0).max(7.0).min(10.0);
    
    painter.text(
        center + egui::vec2(radius / 3.0 + 5.0, 0.0),
        egui::Align2::LEFT_CENTER,
        "60°",
        egui::FontId::monospace(label_font_size),
        egui::Color32::DARK_GRAY
    );
    
    painter.text(
        center + egui::vec2(radius * 2.0 / 3.0 + 5.0, 0.0),
        egui::Align2::LEFT_CENTER,
        "30°",
        egui::FontId::monospace(label_font_size),
        egui::Color32::DARK_GRAY
    );
}
