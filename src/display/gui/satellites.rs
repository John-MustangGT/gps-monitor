// src/display/gui/satellites.rs v1
//! Satellite table rendering and sorting

use crate::gps::GpsData;
use eframe::egui;

use super::app::SatelliteSortColumn;

pub struct SatellitePanel {
    pub sort_column: SatelliteSortColumn,
    pub sort_ascending: bool,
}

impl SatellitePanel {
    pub fn render(&mut self, ui: &mut egui::Ui, data: &GpsData) {
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

        // Calculate scroll area height
        let available_height = ui.available_size().y;
        let reserved_space = 60.0;
        let scroll_height = (available_height - reserved_space).max(100.0).min(available_height * 0.80);
        
        egui::ScrollArea::vertical()
            .max_height(scroll_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_table(ui, data);
            });

        ui.separator();
//        ui.small("💡 Click column headers to sort • Showing satellites above horizon");
    }

    fn render_table(&mut self, ui: &mut egui::Ui, data: &GpsData) {
        // Filter satellites above horizon
        let mut visible_satellites: Vec<_> = data.satellites_info.iter()
            .filter(|sat| sat.elevation.map_or(true, |el| el >= 0.0))
            .collect();
        
        // Sort by selected column
        self.sort_satellites(&mut visible_satellites);

        if visible_satellites.is_empty() {
            ui.weak("No visible satellites");
            return;
        }

        // Create table with clickable headers
        egui::Grid::new("satellite_table")
            .num_columns(7)
            .spacing([8.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                self.render_headers(ui);
                self.render_rows(ui, &visible_satellites);
            });
    }

    fn sort_satellites(&self, satellites: &mut Vec<&crate::gps::data::SatelliteInfo>) {
        match self.sort_column {
            SatelliteSortColumn::Constellation => {
                satellites.sort_by(|a, b| {
                    let cmp = a.constellation.cmp(&b.constellation).then(a.prn.cmp(&b.prn));
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
            SatelliteSortColumn::Prn => {
                satellites.sort_by(|a, b| {
                    let cmp = a.prn.cmp(&b.prn);
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
            SatelliteSortColumn::Used => {
                satellites.sort_by(|a, b| {
                    let cmp = b.used.cmp(&a.used);
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
            SatelliteSortColumn::Snr => {
                satellites.sort_by(|a, b| {
                    let cmp = b.snr.partial_cmp(&a.snr).unwrap_or(std::cmp::Ordering::Equal);
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
            SatelliteSortColumn::Quality => {
                let quality_rank = |quality: &str| -> u8 {
                    match quality {
                        "Excellent" => 0,
                        "Good" => 1,
                        "Fair" => 2,
                        "Poor" => 3,
                        "Very Poor" => 4,
                        _ => 5,
                    }
                };
                
                satellites.sort_by(|a, b| {
                    let a_rank = quality_rank(&a.signal_strength_description());
                    let b_rank = quality_rank(&b.signal_strength_description());
                    let cmp = a_rank.cmp(&b_rank);
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
            SatelliteSortColumn::Elevation => {
                satellites.sort_by(|a, b| {
                    let cmp = b.elevation.partial_cmp(&a.elevation).unwrap_or(std::cmp::Ordering::Equal);
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
            SatelliteSortColumn::Azimuth => {
                satellites.sort_by(|a, b| {
                    let cmp = a.azimuth.partial_cmp(&b.azimuth).unwrap_or(std::cmp::Ordering::Equal);
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
        }
    }

    fn render_headers(&mut self, ui: &mut egui::Ui) {
        let make_header = |ui: &mut egui::Ui, text: &str, column: SatelliteSortColumn, current: SatelliteSortColumn, asc: bool| {
            let arrow = if column == current {
                if asc { " ▲" } else { " ▼" }
            } else {
                ""
            };
            ui.strong(format!("{}{}", text, arrow)).clicked()
        };

        if make_header(ui, "Constellation", SatelliteSortColumn::Constellation, self.sort_column, self.sort_ascending) {
            self.toggle_sort(SatelliteSortColumn::Constellation, true);
        }
        
        if make_header(ui, "PRN", SatelliteSortColumn::Prn, self.sort_column, self.sort_ascending) {
            self.toggle_sort(SatelliteSortColumn::Prn, true);
        }
        
        if make_header(ui, "Used", SatelliteSortColumn::Used, self.sort_column, self.sort_ascending) {
            self.toggle_sort(SatelliteSortColumn::Used, false);
        }
        
        if make_header(ui, "SNR (dB)", SatelliteSortColumn::Snr, self.sort_column, self.sort_ascending) {
            self.toggle_sort(SatelliteSortColumn::Snr, false);
        }
        
        if make_header(ui, "Quality", SatelliteSortColumn::Quality, self.sort_column, self.sort_ascending) {
            self.toggle_sort(SatelliteSortColumn::Quality, true);
        }
        
        if make_header(ui, "Elevation", SatelliteSortColumn::Elevation, self.sort_column, self.sort_ascending) {
            self.toggle_sort(SatelliteSortColumn::Elevation, false);
        }
        
        if make_header(ui, "Azimuth", SatelliteSortColumn::Azimuth, self.sort_column, self.sort_ascending) {
            self.toggle_sort(SatelliteSortColumn::Azimuth, true);
        }
        
        ui.end_row();
    }

    fn toggle_sort(&mut self, column: SatelliteSortColumn, default_ascending: bool) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = default_ascending;
        }
    }

    fn render_rows(&self, ui: &mut egui::Ui, satellites: &[&crate::gps::data::SatelliteInfo]) {
        for sat in satellites {
            // Constellation with symbol
            let symbol = match sat.constellation.as_str() {
                "GPS" => "🇺🇸",
                "GLONASS" => "🇷🇺",
                "GALILEO" => "🇪🇺",
                "BEIDOU" => "🇨🇳",
                "QZSS" => "🇯🇵",
                "SBAS" => "📡",
                _ => "❓",
            };
            ui.label(format!("{} {}", symbol, sat.constellation));

            // PRN
            ui.monospace(format!("{}", sat.prn));

            // Used indicator
            if sat.used {
                ui.colored_label(egui::Color32::GREEN, "✓ Yes");
            } else {
                ui.colored_label(egui::Color32::GRAY, "○ No");
            }

            // SNR with color coding
            if let Some(snr) = sat.snr {
                let color = match snr {
                    s if s >= 40.0 => egui::Color32::GREEN,
                    s if s >= 35.0 => egui::Color32::from_rgb(144, 238, 144),
                    s if s >= 25.0 => egui::Color32::YELLOW,
                    s if s >= 15.0 => egui::Color32::from_rgb(255, 165, 0),
                    _ => egui::Color32::RED,
                };
                ui.colored_label(color, format!("{:.1}", snr));
            } else {
                ui.colored_label(egui::Color32::GRAY, "--");
            }

            // Quality
            let quality_text = sat.signal_strength_description();
            let quality_color = match quality_text.as_str() {
                "Excellent" => egui::Color32::GREEN,
                "Good" => egui::Color32::from_rgb(144, 238, 144),
                "Fair" => egui::Color32::YELLOW,
                "Poor" => egui::Color32::from_rgb(255, 165, 0),
                "Very Poor" => egui::Color32::RED,
                _ => egui::Color32::GRAY,
            };
            ui.colored_label(quality_color, quality_text);

            // Elevation
            if let Some(el) = sat.elevation {
                ui.monospace(format!("{:>3.0}°", el));
            } else {
                ui.colored_label(egui::Color32::GRAY, " --");
            }

            // Azimuth
            if let Some(az) = sat.azimuth {
                ui.monospace(format!("{:>3.0}°", az));
            } else {
                ui.colored_label(egui::Color32::GRAY, " --");
            }

            ui.end_row();
        }
    }
}
