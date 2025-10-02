// src/display/gui/app.rs v10
//! Main GUI application structure - Pure egui implementation

use crate::{gps::GpsData, config::GpsConfig, monitor::{GpsMonitor, GpsSource}, map::TileCache};
use chrono::{DateTime, Utc};
use eframe::egui;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
    path::PathBuf,
};
use tokio::runtime::Runtime;

use super::{panels, satellites::SatellitePanel, skyplot, settings::SettingsWindow, waypoint_dialog::WaypointDialog, map_window::MapWindow};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SatelliteSortColumn {
    Constellation,
    Prn,
    Used,
    Snr,
    Quality,
    Elevation,
    Azimuth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

pub struct GpsGuiApp {
    data: Arc<RwLock<GpsData>>,
    running: Arc<AtomicBool>,
    _last_update: Option<DateTime<Utc>>,
    pub sat_sort_column: SatelliteSortColumn,
    pub sat_sort_ascending: bool,
    settings_window: SettingsWindow,
    waypoint_dialog: WaypointDialog,
    map_window: MapWindow,
    monitor: Option<GpsMonitor>,
    connection_state: ConnectionState,
    error_message: Option<String>,
    config: GpsConfig,
    runtime: Arc<Runtime>,
}

impl GpsGuiApp {
    pub fn new_from_config(config: GpsConfig) -> Self {
        let data = Arc::new(RwLock::new(GpsData::new()));
        let running = Arc::new(AtomicBool::new(false));
        
        // Create Tokio runtime for async operations
        let runtime = Arc::new(
            Runtime::new().expect("Failed to create Tokio runtime")
        );
        
        // Create tile cache directory
        let cache_dir = Self::get_cache_directory();
        let tile_cache = TileCache::new(cache_dir)
            .expect("Failed to create tile cache");
        
        let mut app = Self {
            data,
            running,
            _last_update: None,
            sat_sort_column: SatelliteSortColumn::Constellation,
            sat_sort_ascending: true,
            settings_window: SettingsWindow::new(config.clone()),
            waypoint_dialog: WaypointDialog::new(),
            map_window: MapWindow::new(tile_cache),
            monitor: None,
            connection_state: ConnectionState::Disconnected,
            error_message: None,
            config,
            runtime,
        };
        
        // Auto-connect on startup
        app.start_connection();
        
        app
    }

    fn get_cache_directory() -> PathBuf {
        let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("gps-monitor");
        path.push("tiles");
        path
    }

    fn start_connection(&mut self) {
        self.connection_state = ConnectionState::Connecting;
        self.error_message = None;
        self.running.store(true, Ordering::Relaxed);
        
        let monitor = GpsMonitor::new_with_shared(
            Arc::clone(&self.data),
            Arc::clone(&self.running)
        );
        
        let source = self.create_gps_source();
        
        // Start connection in background using our runtime
        let monitor_clone = monitor.clone();
        let runtime = Arc::clone(&self.runtime);
        std::thread::spawn(move || {
            runtime.block_on(async move {
                if let Err(e) = monitor_clone.start(source).await {
                    eprintln!("Failed to start GPS connection: {}", e);
                }
            });
        });
        
        self.monitor = Some(monitor);
        self.connection_state = ConnectionState::Connected;
    }

    fn stop_connection(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.monitor = None;
        self.connection_state = ConnectionState::Disconnected;
    }

    fn restart_connection(&mut self) {
        self.stop_connection();
        // Small delay to ensure cleanup
        std::thread::sleep(Duration::from_millis(500));
        self.start_connection();
    }

    fn create_gps_source(&self) -> GpsSource {
        match self.config.source_type.as_str() {
            "serial" => {
                let port = self.config.serial_port.clone().unwrap_or_default();
                let baudrate = self.config.serial_baudrate.unwrap_or(9600);
                GpsSource::Serial { port, baudrate }
            }
            "gpsd" => {
                let host = self.config.gpsd_host.clone().unwrap_or_else(|| "localhost".to_string());
                let port = self.config.gpsd_port.unwrap_or(2947);
                GpsSource::Gpsd { host, port }
            }
            #[cfg(windows)]
            "windows" => {
                let accuracy = self.config.windows_accuracy.unwrap_or(10);
                let interval = self.config.windows_interval.unwrap_or(1);
                GpsSource::Windows { accuracy, interval }
            }
            _ => {
                // Default to platform-specific source
                #[cfg(windows)]
                {
                    GpsSource::Windows { accuracy: 10, interval: 1 }
                }
                #[cfg(not(windows))]
                {
                    GpsSource::Gpsd {
                        host: "localhost".to_string(),
                        port: 2947,
                    }
                }
            }
        }
    }

    fn render_top_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("🛰 GPS Monitor");
                ui.separator();
                
                // Connection state indicator
                let (status_color, status_text) = match self.connection_state {
                    ConnectionState::Connected => {
                        let data = self.data.read().unwrap();
                        if data.timestamp.is_some() && data.is_recent() {
                            (egui::Color32::GREEN, "Connected")
                        } else {
                            (egui::Color32::YELLOW, "Waiting for data")
                        }
                    }
                    ConnectionState::Connecting => (egui::Color32::YELLOW, "Connecting..."),
                    ConnectionState::Disconnected => (egui::Color32::RED, "Disconnected"),
                };
                
                ui.colored_label(status_color, "●");
                ui.label(status_text);
                
                // Last update timestamp
                let data = self.data.read().unwrap();
                let timestamp_str = match data.timestamp {
                    Some(ts) => ts.format("%H:%M:%S UTC").to_string(),
                    None => "No data".to_string(),
                };
                ui.label(format!("Last Update: {}", timestamp_str));
                
                if let Some(ref source) = data.source {
                    ui.separator();
                    ui.label(format!("Source: {}", source));
                }
                drop(data);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ Exit").clicked() {
                        self.stop_connection();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    
                    if ui.button("⚙ Settings").clicked() {
                        self.settings_window.open = true;
                    }

                    if ui.button("📍 Waypoints").clicked() {
                        self.waypoint_dialog.open = true;
                    }
                    
                    if ui.button("🗺 Map").clicked() {
                        self.map_window.open = true;
                    }
                    
                    if ui.button("🔄 Restart").clicked() {
                        self.restart_connection();
                    }
                    
                    // Connection control
                    match self.connection_state {
                        ConnectionState::Connected | ConnectionState::Connecting => {
                            if ui.button("⏸ Disconnect").clicked() {
                                self.stop_connection();
                            }
                        }
                        ConnectionState::Disconnected => {
                            if ui.button("▶ Connect").clicked() {
                                self.start_connection();
                            }
                        }
                    }
                });
            });
        });
    }

    fn render_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(80.0)
            .show(ctx, |ui| {
                ui.label("📝 Latest NMEA Sentences / Raw Data");
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(60.0).show(ui, |ui| {
                    let data = self.data.read().unwrap();
                    if !data.raw_history.is_empty() {
                        for sentence in data.raw_history.iter().rev() {
                            ui.monospace(sentence);
                        }
                    } else if !data.raw_data.is_empty() {
                        ui.monospace(&data.raw_data);
                    } else {
                        ui.weak("No data received");
                    }
                });
            });
    }

    fn render_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            
            ui.horizontal(|ui| {
                // Left panel - Main GPS data (40% of width)
                let left_width = available_size.x * 0.4;
                ui.allocate_ui_with_layout(
                    [left_width, available_size.y].into(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.group(|ui| {
                            ui.set_width(left_width - 10.0);
                            ui.set_height(available_size.y - 10.0);
                            
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let data = self.data.read().unwrap();
                                panels::render_main_data_panel(ui, &data);
                            });
                        });
                    }
                );

                ui.separator();

                // Right panel - Sky plot and satellites (60% of width)
                let right_width = available_size.x * 0.6 - 20.0;
                ui.allocate_ui_with_layout(
                    [right_width, available_size.y].into(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let sky_plot_height = (available_size.y * 0.5).max(200.0).min(400.0);
                        let satellite_table_height = available_size.y - sky_plot_height - 20.0;
                        
                        // Sky plot (top section)
                        ui.group(|ui| {
                            ui.set_width(right_width - 10.0);
                            ui.set_height(sky_plot_height);
                            let data = self.data.read().unwrap();
                            skyplot::render_sky_plot(ui, &data);
                        });

                        ui.add_space(5.0);

                        // Satellite table (bottom section)
                        ui.group(|ui| {
                            ui.set_width(right_width - 10.0);
                            ui.set_height(satellite_table_height.max(150.0));
                            
                            let data = self.data.read().unwrap();
                            let mut sat_panel = SatellitePanel {
                                sort_column: self.sat_sort_column,
                                sort_ascending: self.sat_sort_ascending,
                            };
                            sat_panel.render(ui, &data);
                            
                            // Update sort state from panel
                            self.sat_sort_column = sat_panel.sort_column;
                            self.sat_sort_ascending = sat_panel.sort_ascending;
                        });
                    }
                );
            });
        });
    }

    fn handle_settings_window(&mut self, ctx: &egui::Context) {
        if self.settings_window.show(ctx) {
            // Configuration was saved, reload it
            self.config = self.settings_window.get_config().clone();
            
            // Ask user if they want to reconnect
            self.error_message = Some("Settings saved! Click 'Restart' to apply changes.".to_string());
        }
    }

    fn handle_waypoint_dialog(&mut self, ctx: &egui::Context) {
        let data = self.data.read().unwrap().clone();
        self.waypoint_dialog.show(ctx, &data);
    }

    fn handle_map_window(&mut self, ctx: &egui::Context) {
        let data = self.data.read().unwrap().clone();
        self.map_window.show(ctx, &data, &self.waypoint_dialog.exporter);
        
        // Clean up when window closes
        if !self.map_window.open {
            self.map_window.on_close();
        }
    }

    fn show_error_notification(&mut self, ctx: &egui::Context) {
        // Take ownership of error_message to avoid borrow issues
        if let Some(msg) = self.error_message.take() {
            let mut keep_showing = true;
            
            egui::Window::new("ℹ Notification")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(&msg);
                    if ui.button("OK").clicked() {
                        keep_showing = false;
                    }
                });
            
            // Put the message back if we should keep showing it
            if keep_showing {
                self.error_message = Some(msg);
            }
        }
    }
}

impl eframe::App for GpsGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint every second
        ctx.request_repaint_after(Duration::from_secs(1));

        // Render UI components
        self.render_top_menu(ctx);
        self.render_bottom_panel(ctx);
        self.render_main_content(ctx);
        self.handle_settings_window(ctx);
        self.handle_waypoint_dialog(ctx);
        self.handle_map_window(ctx);
        self.show_error_notification(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_connection();
    }
}
