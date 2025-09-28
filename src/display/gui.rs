// src/display/gui.rs
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
        let (tx, rx) = mpsc::channel();
        let data_clone = Arc::clone(&data);
        let running_clone = Arc::clone(&running);

        // Spawn GUI thread
        let gui_handle = std::thread::spawn(move || {
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([800.0, 600.0])
                    .with_title("GPS Monitor"),
                ..Default::default()
            };

            let app = GpsGuiApp::new(data_clone, running_clone, tx);
            eframe::run_native("GPS Monitor", options, Box::new(|_cc| Ok(Box::new(app))))
        });

        // Wait for GUI to signal shutdown or handle Ctrl+C
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                running.store(false, Ordering::Relaxed);
            }
            _ = tokio::task::spawn_blocking(move || {
                let _ = rx.recv(); // Wait for GUI to close
            }) => {
                running.store(false, Ordering::Relaxed);
            }
        }

        // Wait for GUI thread to complete
        gui_handle.join().map_err(|_| GpsError::Other("GUI thread panicked".to_string()))?;
        Ok(())
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
}

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl eframe::App for GpsGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint every second
        ctx.request_repaint_after(Duration::from_secs(1));

        let data = self.data.read().unwrap().clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🛰 GPS Monitor");
            ui.separator();

            // Status bar
            ui.horizontal(|ui| {
                let status_color = if data.timestamp.is_some() && data.is_recent() {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                };

                ui.colored_label(status_color, "●");

                let timestamp_str = match data.timestamp {
                    Some(ts) => ts.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    None => "No data received".to_string(),
                };
                ui.label(format!("Last Update: {}", timestamp_str));

                if let Some(ref source) = data.source {
                    ui.separator();
                    ui.label(format!("Source: {}", source));
                }
            });

            ui.separator();

            // Position section
            ui.collapsing("📍 Position", |ui| {
                egui::Grid::new("position_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
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
            });

            // Movement section
            ui.collapsing("🧭 Movement", |ui| {
                egui::Grid::new("movement_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Speed:");
                        ui.monospace(Self::format_value(data.speed, "km/h"));
                        ui.end_row();

                        ui.label("Course:");
                        ui.monospace(Self::format_value(data.course, "°"));
                        ui.end_row();
                    });
            });

            // Quality section (if GPS data available)
            if data.satellites.is_some() || data.hdop.is_some() || data.fix_quality.is_some() {
                ui.collapsing("📡 Signal Quality", |ui| {
                    egui::Grid::new("quality_grid")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
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
                });
            }

            // Raw data section
            ui.collapsing("📝 Raw Data", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Data:");
                    if !data.raw_data.is_empty() {
                        ui.monospace(&data.raw_data);
                    } else {
                        ui.weak("No data");
                    }
                });
            });

            ui.separator();

            // Control buttons
            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh").clicked() {
                    // Force refresh - data updates automatically
                }

                ui.separator();

                if ui.button("❌ Exit").clicked() {
                    self.running.store(false, Ordering::Relaxed);
                    let _ = self.shutdown_tx.send(());
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
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
