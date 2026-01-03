// src/display/gui/imu_panel.rs
//! IMU visualization panel with car representation

use crate::gps::GpsData;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::f64::consts::PI;

/// Render the IMU panel showing vehicle dynamics
pub fn render_imu_panel(ui: &mut egui::Ui, data: &GpsData) {
    ui.heading("🏎 Vehicle Dynamics");
    ui.add_space(5.0);

    // Extract IMU data
    let (gx, gy, gz) = data.acceleration.unwrap_or((0.0, 0.0, 1.0));
    let (rx, ry, rz) = data.rotation.unwrap_or((0.0, 0.0, 0.0));

    // Display numeric values
    render_numeric_values(ui, gx, gy, gz, rx, ry, rz);

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Render visual car representation
    render_car_visualization(ui, gx, gy, gz, rx, ry, rz);
}

/// Render numeric IMU values in a grid
fn render_numeric_values(ui: &mut egui::Ui, gx: f64, gy: f64, gz: f64, rx: f64, ry: f64, rz: f64) {
    ui.label("Accelerometer (g-force):");
    egui::Grid::new("imu_accel_grid")
        .num_columns(3)
        .spacing([15.0, 8.0])
        .show(ui, |ui| {
            ui.label(format!("X: {:>7.3}", gx));
            ui.label(format!("Y: {:>7.3}", gy));
            ui.label(format!("Z: {:>7.3}", gz));
        });

    ui.add_space(5.0);

    ui.label("Gyroscope (deg/s):");
    egui::Grid::new("imu_gyro_grid")
        .num_columns(3)
        .spacing([15.0, 8.0])
        .show(ui, |ui| {
            ui.label(format!("Roll:  {:>7.2}", rx));
            ui.label(format!("Pitch: {:>7.2}", ry));
            ui.label(format!("Yaw:   {:>7.2}", rz));
        });

    ui.add_space(5.0);

    // Calculate total g-force and turn rate
    let total_g = (gx * gx + gy * gy + gz * gz).sqrt();
    let turn_rate = rz.abs();

    ui.horizontal(|ui| {
        ui.label(format!("Total G-Force: {:.3}", total_g));
        ui.separator();
        ui.label(format!("Turn Rate: {:.1}°/s", turn_rate));
    });
}

/// Render car visualization with g-force vectors and rotation
fn render_car_visualization(ui: &mut egui::Ui, gx: f64, gy: f64, _gz: f64, _rx: f64, _ry: f64, rz: f64) {
    // Request a square canvas for the car visualization
    let desired_size = Vec2::new(400.0, 400.0);
    let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
    let rect = response.rect;
    let center = rect.center();

    // Draw background
    painter.rect_filled(rect, 3.0, Color32::from_rgb(20, 20, 25));

    // Car dimensions (top-down view)
    let car_width = 60.0;
    let car_height = 100.0;
    let car_rect = Rect::from_center_size(
        center,
        Vec2::new(car_width, car_height),
    );

    // Draw car body (top-down view)
    painter.rect_filled(car_rect, 3.0, Color32::from_rgb(60, 80, 120));
    painter.rect_stroke(car_rect, 3.0, Stroke::new(2.0, Color32::from_rgb(100, 120, 160)));

    // Draw car front indicator (top of rectangle)
    let front_indicator_y = car_rect.min.y;
    let front_indicator_start = Pos2::new(car_rect.center().x - 10.0, front_indicator_y);
    let front_indicator_end = Pos2::new(car_rect.center().x + 10.0, front_indicator_y);
    painter.line_segment(
        [front_indicator_start, front_indicator_end],
        Stroke::new(4.0, Color32::from_rgb(255, 200, 0)),
    );

    // Draw grid reference lines
    draw_grid(&painter, center, rect);

    // Scale factors for visualization
    let g_scale = 80.0; // pixels per g-force
    let rotation_scale = 3.0; // arc size multiplier

    // Draw acceleration vector (g-force arrow)
    // Note: In top-down view, gx is lateral (left-right), gy is longitudinal (forward-back)
    let g_arrow_end = Pos2::new(
        center.x + (gx * g_scale) as f32,
        center.y - (gy * g_scale) as f32, // Negative because screen Y increases downward
    );

    let g_magnitude = (gx * gx + gy * gy).sqrt();
    if g_magnitude > 0.05 {
        draw_arrow(
            &painter,
            center,
            g_arrow_end,
            Color32::from_rgb(255, 100, 100),
            "G-Force",
        );
    }

    // Draw rotation indicator (yaw/turn rate)
    // Positive rz = turning right (clockwise), negative = turning left (counter-clockwise)
    if rz.abs() > 5.0 {
        let rotation_color = if rz > 0.0 {
            Color32::from_rgb(100, 255, 100) // Green for right turn
        } else {
            Color32::from_rgb(255, 255, 100) // Yellow for left turn
        };

        draw_rotation_arc(&painter, center, 120.0, rz, rotation_color, rotation_scale);
    }

    // Draw legend
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.small("Legend:");
        ui.colored_label(Color32::from_rgb(255, 100, 100), "● G-Force");
        ui.colored_label(Color32::from_rgb(100, 255, 100), "● Turn Right");
        ui.colored_label(Color32::from_rgb(255, 255, 100), "● Turn Left");
    });
}

/// Draw a grid for reference
fn draw_grid(painter: &egui::Painter, center: Pos2, rect: Rect) {
    let grid_color = Color32::from_rgb(40, 40, 45);
    let axis_color = Color32::from_rgb(80, 80, 90);

    // Draw grid lines every 50 pixels
    let grid_spacing = 50.0;
    let mut x = center.x;
    while x < rect.max.x {
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0, grid_color),
        );
        x += grid_spacing;
    }
    x = center.x - grid_spacing;
    while x > rect.min.x {
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0, grid_color),
        );
        x -= grid_spacing;
    }

    let mut y = center.y;
    while y < rect.max.y {
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0, grid_color),
        );
        y += grid_spacing;
    }
    y = center.y - grid_spacing;
    while y > rect.min.y {
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0, grid_color),
        );
        y -= grid_spacing;
    }

    // Draw center axes
    painter.line_segment(
        [Pos2::new(rect.min.x, center.y), Pos2::new(rect.max.x, center.y)],
        Stroke::new(2.0, axis_color),
    );
    painter.line_segment(
        [Pos2::new(center.x, rect.min.y), Pos2::new(center.x, rect.max.y)],
        Stroke::new(2.0, axis_color),
    );
}

/// Draw an arrow from start to end
fn draw_arrow(painter: &egui::Painter, start: Pos2, end: Pos2, color: Color32, _label: &str) {
    // Draw main line
    painter.line_segment([start, end], Stroke::new(3.0, color));

    // Calculate arrowhead
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();

    if length > 5.0 {
        let angle = dy.atan2(dx);
        let arrow_size = 12.0;
        let arrow_angle = PI as f32 / 6.0; // 30 degrees

        let arrow_left = Pos2::new(
            end.x - arrow_size * (angle - arrow_angle).cos(),
            end.y - arrow_size * (angle - arrow_angle).sin(),
        );
        let arrow_right = Pos2::new(
            end.x - arrow_size * (angle + arrow_angle).cos(),
            end.y - arrow_size * (angle + arrow_angle).sin(),
        );

        painter.line_segment([end, arrow_left], Stroke::new(3.0, color));
        painter.line_segment([end, arrow_right], Stroke::new(3.0, color));
    }
}

/// Draw rotation arc indicator
fn draw_rotation_arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    rz: f64,
    color: Color32,
    scale: f64,
) {
    // Calculate arc angle based on rotation rate
    let arc_angle = (rz.abs() * scale).min(180.0) as f32;
    let arc_radians = arc_angle * PI as f32 / 180.0;

    // Determine if rotation is clockwise (right) or counter-clockwise (left)
    let clockwise = rz > 0.0;

    // Draw rotation arc
    let num_segments = 32;
    let start_angle = if clockwise {
        -PI as f32 / 2.0 // Start from top, go clockwise
    } else {
        -PI as f32 / 2.0 - arc_radians // Start from top minus arc, go counter-clockwise
    };

    for i in 0..num_segments {
        let t1 = i as f32 / num_segments as f32;
        let t2 = (i + 1) as f32 / num_segments as f32;

        let angle1 = start_angle + t1 * arc_radians;
        let angle2 = start_angle + t2 * arc_radians;

        let p1 = Pos2::new(
            center.x + radius * angle1.cos(),
            center.y + radius * angle1.sin(),
        );
        let p2 = Pos2::new(
            center.x + radius * angle2.cos(),
            center.y + radius * angle2.sin(),
        );

        painter.line_segment([p1, p2], Stroke::new(4.0, color));
    }

    // Draw arrow at end of arc to indicate direction
    let arrow_angle = if clockwise {
        start_angle + arc_radians
    } else {
        start_angle
    };

    let arrow_pos = Pos2::new(
        center.x + radius * arrow_angle.cos(),
        center.y + radius * arrow_angle.sin(),
    );

    // Arrowhead perpendicular to radius
    let tangent_angle = if clockwise {
        arrow_angle + PI as f32 / 2.0
    } else {
        arrow_angle - PI as f32 / 2.0
    };

    let arrow_size = 10.0;
    let arrow_left = Pos2::new(
        arrow_pos.x + arrow_size * (tangent_angle - 0.3).cos(),
        arrow_pos.y + arrow_size * (tangent_angle - 0.3).sin(),
    );
    let arrow_right = Pos2::new(
        arrow_pos.x + arrow_size * (tangent_angle + 0.3).cos(),
        arrow_pos.y + arrow_size * (tangent_angle + 0.3).sin(),
    );

    painter.line_segment([arrow_pos, arrow_left], Stroke::new(4.0, color));
    painter.line_segment([arrow_pos, arrow_right], Stroke::new(4.0, color));
}
