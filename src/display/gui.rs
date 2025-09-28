// src/display/gui.rs v13
//! GUI display implementation using egui

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
use {
    crate::{
        gps::GpsData,
        error::{Result, GpsError},
    },
    chrono::{DateTime, Utc},
    eframe::egui,
    std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc, Arc, RwLock,
        },
        time::Duration,
    },
};

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub struct GuiDisplay;

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl GuiDisplay {
    pub fn new() -> Self {
        Self
    }

    /// Start the GUI display
    pub async fn run(
        &self,
        data: Arc<RwLock<GpsData>>,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        let (tx, _rx) = mpsc::channel();
        let data_clone = Arc::clone(&data);
        let running_clone = Arc::clone(&running);

        // Spawn GUI thread (eframe must run on main thread, so we'll handle this differently)
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_title("GPS Monitor"),
            ..Default::default()
        };

        let app = GpsGuiApp::new(data_clone, running_clone, tx);
        
        // Run eframe - this blocks until the window is closed
        match eframe::run_native("GPS Monitor", options, Box::new(|_cc| Ok(Box::new(app)))) {
            Ok(_) => Ok(()),
            Err(e) => Err(GpsError::Other(format!("GUI error: {}", e))),
        }
    }
}

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl Default for GuiDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub struct GpsGuiApp {
    data: Arc<RwLock<GpsData>>,
    running: Arc<AtomicBool>,
    shutdown_tx: mpsc::Sender<()>,
    _last_update: Option<DateTime<Utc>>,
}

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl GpsGuiApp {
    pub fn new(
        data: Arc<RwLock<GpsData>>,
        running: Arc<AtomicBool>,
        shutdown_tx: mpsc::Sender<()>,
    ) -> Self {
        Self {
            data,
            running,
            shutdown_tx,
            _last_update: None,
        }
    }

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

    fn render_main_data_panel(&self, ui: &mut egui::Ui, data: &GpsData) {
        ui.strong("📍 Position & Movement");
        ui.separator();

        // Position section
        egui::Grid::new("position_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Latitude:");
                ui.monospace(Self::format_coordinate(data.latitude));
                ui.end_row();

                ui.label("Longitude:");
                ui.monospace(Self::format_coordinate(data.longitude));
                ui.end_row();

                ui.label("Altitude:");
                ui.monospace(Self::format_value(data.altitude, "m"));
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
                ui.monospace(Self::format_value(data.speed, "km/h"));
                ui.end_row();

                ui.label("Course:");
                ui.monospace(Self::format_value(data.course, "°"));
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

    fn render_satellite_panel(&self, ui: &mut egui::Ui, data: &GpsData) {
        ui.strong("🛰 Satellites");
        ui.separator();

        if data.satellites_info.is_empty() {
            ui.weak("No satellite data available");
            return;
        }

        // Summary
        let used_count = data.satellites_used();
        let total_count = data.satellites_info.len();
        ui.label(format!("📊 {} used / {} visible", used_count, total_count));
        ui.add_space(5.0);

        // Satellite list in a scrollable area
        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            // Group satellites by constellation
            let grouped_sats = data.satellites_by_constellation();
            
            for (constellation, satellites) in grouped_sats {
                // Constellation header with symbol
                let constellation_symbol = match constellation.as_str() {
                    "GPS" => "🇺🇸",
                    "GLONASS" => "🇷🇺", 
                    "GALILEO" => "🇪🇺",
                    "BEIDOU" => "🇨🇳",
                    "QZSS" => "🇯🇵",
                    "SBAS" => "📡",
                    _ => "❓",
                };
                
                ui.strong(format!("{} {} ({})", constellation_symbol, constellation, satellites.len()));
                
                // Sort satellites by PRN
                let mut sorted_sats = satellites.clone();
                sorted_sats.sort_by_key(|sat| sat.prn);
                
                // Show satellites in a compact grid
                ui.group(|ui| {
                    let mut current_row_count = 0;
                    const SATS_PER_ROW: usize = 2;
                    
                    ui.horizontal_wrapped(|ui| {
                        for sat in sorted_sats {
                            if current_row_count >= SATS_PER_ROW {
                                ui.end_row();
                                current_row_count = 0;
                            }
                            
                            // Satellite card
                            ui.group(|ui| {
                                ui.set_min_width(140.0);
                                
                                // PRN and usage status
                                ui.horizontal(|ui| {
                                    ui.strong(format!("PRN {}", sat.prn));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if sat.used {
                                            ui.colored_label(egui::Color32::GREEN, "●");
                                        } else {
                                            ui.colored_label(egui::Color32::GRAY, "○");
                                        }
                                    });
                                });
                                
                                // Signal strength with color coding
                                if let Some(snr) = sat.snr {
                                    let (color, strength_text) = match snr {
                                        s if s >= 40.0 => (egui::Color32::GREEN, "Excellent"),
                                        s if s >= 35.0 => (egui::Color32::from_rgb(144, 238, 144), "Good"),
                                        s if s >= 25.0 => (egui::Color32::YELLOW, "Fair"),
                                        s if s >= 15.0 => (egui::Color32::from_rgb(255, 165, 0), "Poor"),
                                        _ => (egui::Color32::RED, "Very Poor"),
                                    };
                                    
                                    ui.horizontal(|ui| {
                                        ui.colored_label(color, format!("{:.0} dB", snr));
                                        ui.small(strength_text);
                                    });
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, "No signal");
                                }
                                
                                // Position info (compact)
                                if let (Some(el), Some(az)) = (sat.elevation, sat.azimuth) {
                                    ui.small(format!("El: {:.0}° Az: {:.0}°", el, az));
                                } else {
                                    ui.small("Position: Unknown");
                                }
                            });
                            
                            current_row_count += 1;
                        }
                    });
                });
                
                ui.add_space(5.0);
            }
        });

        ui.separator();
        ui.small("💡 ● = Used in fix, ○ = Visible only");
    }

    fn render_sky_plot(&self, ui: &mut egui::Ui, data: &GpsData) {
        ui.strong("🌌 Sky Plot");
        ui.separator();

        if data.satellites_info.is_empty() {
            ui.weak("No satellite position data");
            return;
        }

        // Calculate the plot area
        let available_size = ui.available_size();
        let plot_size = available_size.x.min(available_size.y - 30.0).min(220.0);
        let _center = egui::pos2(plot_size / 2.0, plot_size / 2.0);
        let radius = plot_size / 2.0 - 20.0; // Leave margin for labels

        // Reserve space for the plot
        let (rect, _response) = ui.allocate_exact_size(
            [plot_size, plot_size].into(),
            egui::Sense::hover()
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            
            // Draw background circle (horizon)
            painter.circle_stroke(
                rect.center(),
                radius,
                egui::Stroke::new(2.0, egui::Color32::GRAY)
            );

            // Draw elevation circles (30°, 60°)
            painter.circle_stroke(
                rect.center(),
                radius * 2.0 / 3.0, // 60° elevation
                egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
            );
            painter.circle_stroke(
                rect.center(),
                radius / 3.0, // 30° elevation
                egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
            );

            // Draw cardinal direction lines and labels
            let directions: [(f32, &str); 4] = [
                (0.0, "N"),   // North (top)
                (90.0, "E"),  // East (right)
                (180.0, "S"), // South (bottom)
                (270.0, "W"), // West (left)
            ];

            for (angle_deg, label) in directions {
                let angle_rad = angle_deg.to_radians();
                let end_pos = rect.center() + egui::vec2(
                    angle_rad.sin() * radius,
                    -angle_rad.cos() * radius // Negative because screen Y increases downward
                );
                
                // Draw direction line
                painter.line_segment(
                    [rect.center(), end_pos],
                    egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
                );

                // Draw direction label
                let label_pos = rect.center() + egui::vec2(
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

            // Plot satellites
            for sat in &data.satellites_info {
                if let (Some(elevation), Some(azimuth)) = (sat.elevation, sat.azimuth) {
                    // Convert to screen coordinates
                    // Elevation: 90° = center, 0° = edge
                    let elev_normalized = (90.0 - elevation) / 90.0; // 0 = center, 1 = edge
                    let sat_radius = radius * elev_normalized;
                    
                    // Azimuth: 0° = North (top), increases clockwise
                    let azimuth_rad = azimuth.to_radians();
                    let sat_pos = rect.center() + egui::vec2(
                        azimuth_rad.sin() * sat_radius,
                        -azimuth_rad.cos() * sat_radius
                    );

                    // Determine satellite color based on constellation and signal
                    let (sat_color, sat_size) = if sat.used {
                        // Used satellites are larger and more colorful
                        let color = match sat.constellation.as_str() {
                            "GPS" => egui::Color32::from_rgb(0, 150, 255),      // Blue
                            "GLONASS" => egui::Color32::from_rgb(255, 100, 100), // Red
                            "GALILEO" => egui::Color32::from_rgb(100, 255, 100), // Green
                            "BEIDOU" => egui::Color32::from_rgb(255, 255, 100),  // Yellow
                            "QZSS" => egui::Color32::from_rgb(255, 150, 0),     // Orange
                            _ => egui::Color32::WHITE,
                        };
                        (color, 8.0)
                    } else {
                        // Unused satellites are smaller and gray
                        (egui::Color32::GRAY, 5.0)
                    };

                    // Draw satellite dot
                    painter.circle_filled(sat_pos, sat_size, sat_color);

                    // Draw PRN number next to satellite
                    let text_pos = sat_pos + egui::vec2(sat_size + 2.0, 0.0);
                    painter.text(
                        text_pos,
                        egui::Align2::LEFT_CENTER,
                        sat.prn.to_string(),
                        egui::FontId::monospace(9.0),
                        egui::Color32::WHITE
                    );

                    // Show signal strength as ring for used satellites
                    if sat.used {
                        if let Some(snr) = sat.snr {
                            let ring_color = match snr {
                                s if s >= 40.0 => egui::Color32::GREEN,
                                s if s >= 35.0 => egui::Color32::YELLOW,
                                s if s >= 25.0 => egui::Color32::from_rgb(255, 165, 0), // Orange
                                _ => egui::Color32::RED,
                            };
                            painter.circle_stroke(
                                sat_pos,
                                sat_size + 2.0,
                                egui::Stroke::new(1.5, ring_color)
                            );
                        }
                    }
                }
            }

            // Add elevation labels
            painter.text(
                rect.center() + egui::vec2(radius / 3.0 + 5.0, 0.0),
                egui::Align2::LEFT_CENTER,
                "60°",
                egui::FontId::monospace(8.0),
                egui::Color32::DARK_GRAY
            );
            painter.text(
                rect.center() + egui::vec2(radius * 2.0 / 3.0 + 5.0, 0.0),
                egui::Align2::LEFT_CENTER,
                "30°",
                egui::FontId::monospace(8.0),
                egui::Color32::DARK_GRAY
            );
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
}

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl eframe::App for GpsGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint every second
        ctx.request_repaint_after(Duration::from_secs(1));

        let data = self.data.read().unwrap().clone();

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("🛰 GPS Monitor");
                ui.separator();
                
                // Status indicator
                let status_color = if data.timestamp.is_some() && data.is_recent() {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                };
                ui.colored_label(status_color, "●");
                
                let timestamp_str = match data.timestamp {
                    Some(ts) => ts.format("%H:%M:%S UTC").to_string(),
                    None => "No data".to_string(),
                };
                ui.label(format!("Last Update: {}", timestamp_str));
                
                if let Some(ref source) = data.source {
                    ui.separator();
                    ui.label(format!("Source: {}", source));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ Exit").clicked() {
                        self.running.store(false, Ordering::Relaxed);
                        let _ = self.shutdown_tx.send(());
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // Bottom panel for raw data
        egui::TopBottomPanel::bottom("bottom_panel").resizable(true).default_height(80.0).show(ctx, |ui| {
            ui.label("📝 Latest NMEA Sentences");
            ui.separator();
            
            egui::ScrollArea::vertical().max_height(60.0).show(ui, |ui| {
                if !data.raw_history.is_empty() {
                    // Show the last few sentences in reverse order (newest first)
                    for sentence in data.raw_history.iter().rev() {
                        ui.monospace(sentence);
                    }
                } else if !data.raw_data.is_empty() {
                    // Fallback to showing just the current sentence
                    ui.monospace(&data.raw_data);
                } else {
                    ui.weak("No data received");
                }
            });
        });

        // Main content area with 3 sections
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left column - Main GPS data (40% width)
                ui.allocate_ui_with_layout(
                    [ui.available_width() * 0.4, ui.available_height()].into(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.group(|ui| {
                            ui.set_min_height(400.0);
                            self.render_main_data_panel(ui, &data);
                        });
                    }
                );

                ui.separator();

                // Right side - Split into sky plot (top) and satellite list (bottom)
                ui.allocate_ui_with_layout(
                    [ui.available_width(), ui.available_height()].into(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        // Sky plot (top half)
                        ui.group(|ui| {
                            ui.set_height(250.0);
                            self.render_sky_plot(ui, &data);
                        });

                        ui.add_space(5.0);

                        // Satellite list (bottom half)
                        ui.group(|ui| {
                            ui.set_min_height(200.0);
                            self.render_satellite_panel(ui, &data);
                        });
                    }
                );
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.shutdown_tx.send(());
    }
}

// Stub implementations for non-GUI builds
#[cfg(not(all(unix, not(target_os = "macos"), feature = "gui")))]
pub struct GuiDisplay;

#[cfg(not(all(unix, not(target_os = "macos"), feature = "gui")))]
impl GuiDisplay {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(
        &self,
        _data: Arc<RwLock<GpsData>>,
        _running: Arc<AtomicBool>,
    ) -> Result<()> {
        Err(GpsError::Other("GUI support not compiled in".to_string()))
    }
}
