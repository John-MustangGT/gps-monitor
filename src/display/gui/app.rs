// src/display/gui/app.rs v1
//! Main GUI application structure and eframe::App implementation

use crate::gps::GpsData;
use chrono::{DateTime, Utc};
use eframe::egui;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, RwLock,
    },
    time::Duration,
};

use super::{panels, satellites::SatellitePanel, skyplot};

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

pub struct GpsGuiApp {
    data: Arc<RwLock<GpsData>>,
    running: Arc<AtomicBool>,
    shutdown_tx: mpsc::Sender<()>,
    _last_update: Option<DateTime<Utc>>,
    pub sat_sort_column: SatelliteSortColumn,
    pub sat_sort_ascending: bool,
}

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
            sat_sort_column: SatelliteSortColumn::Constellation,
            sat_sort_ascending: true,
        }
    }
}

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

        // Main content area with flexible layout
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
                            skyplot::render_sky_plot(ui, &data);
                        });

                        ui.add_space(5.0);

                        // Satellite table (bottom section)
                        ui.group(|ui| {
                            ui.set_width(right_width - 10.0);
                            ui.set_height(satellite_table_height.max(150.0));
                            
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.shutdown_tx.send(());
    }
}
