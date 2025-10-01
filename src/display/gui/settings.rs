// src/display/gui/settings.rs v2
//! Settings UI for GPS source configuration

use crate::config::GpsConfig;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceType {
    Serial,
    Gpsd,
    #[cfg(windows)]
    Windows,
}

pub struct SettingsWindow {
    pub open: bool,
    pub config: GpsConfig,
    source_type: SourceType,
    // Temporary UI state
    serial_port: String,
    serial_baudrate: String,
    gpsd_host: String,
    gpsd_port: String,
    #[cfg(windows)]
    windows_accuracy: String,
    #[cfg(windows)]
    windows_interval: String,
    status_message: Option<String>,
}

impl SettingsWindow {
    pub fn new(config: GpsConfig) -> Self {
        let source_type = match config.source_type.as_str() {
            "serial" => SourceType::Serial,
            "gpsd" => SourceType::Gpsd,
            #[cfg(windows)]
            "windows" => SourceType::Windows,
            _ => {
                #[cfg(windows)]
                {
                    SourceType::Windows
                }
                #[cfg(not(windows))]
                {
                    SourceType::Gpsd
                }
            }
        };

        Self {
            open: false,
            serial_port: config.serial_port.clone().unwrap_or_default(),
            serial_baudrate: config.serial_baudrate.map_or("9600".to_string(), |b| b.to_string()),
            gpsd_host: config.gpsd_host.clone().unwrap_or_else(|| "localhost".to_string()),
            gpsd_port: config.gpsd_port.map_or("2947".to_string(), |p| p.to_string()),
            #[cfg(windows)]
            windows_accuracy: config.windows_accuracy.map_or("10".to_string(), |a| a.to_string()),
            #[cfg(windows)]
            windows_interval: config.windows_interval.map_or("1".to_string(), |i| i.to_string()),
            config,
            source_type,
            status_message: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        if !self.open {
            return false;
        }

        let mut config_changed = false;
        
        // We need to avoid .open() because it creates a borrow conflict
        // Instead, we'll manually handle the close button
        let window = egui::Window::new("⚙ Settings")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0);
            
        let response = window.show(ctx, |ui| {
                ui.heading("GPS Source Configuration");
                ui.separator();

                // Source type selection
                ui.label("Select GPS Source:");
                ui.horizontal(|ui| {
                    if ui.radio_value(&mut self.source_type, SourceType::Serial, "Serial Port").clicked() {
                        self.status_message = None;
                    }
                    if ui.radio_value(&mut self.source_type, SourceType::Gpsd, "gpsd").clicked() {
                        self.status_message = None;
                    }
                    #[cfg(windows)]
                    if ui.radio_value(&mut self.source_type, SourceType::Windows, "Windows Location").clicked() {
                        self.status_message = None;
                    }
                });

                ui.add_space(10.0);

                // Configuration fields based on source type
                match self.source_type {
                    SourceType::Serial => {
                        self.render_serial_settings(ui);
                    }
                    SourceType::Gpsd => {
                        self.render_gpsd_settings(ui);
                    }
                    #[cfg(windows)]
                    SourceType::Windows => {
                        self.render_windows_settings(ui);
                    }
                }

                ui.add_space(10.0);
                ui.separator();

                // Status message
                if let Some(ref msg) = self.status_message {
                    ui.colored_label(egui::Color32::GREEN, msg);
                    ui.add_space(5.0);
                }

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("💾 Save & Apply").clicked() {
                        if self.validate_and_save() {
                            config_changed = true;
                            self.status_message = Some("Settings saved successfully!".to_string());
                        }
                    }

                    if ui.button("❌ Cancel").clicked() {
                        self.open = false;
                        self.status_message = None;
                    }
                });

                ui.add_space(5.0);
                ui.separator();
                ui.small("💡 Changes will take effect after restarting the GPS connection");
            });

        // Check if user clicked outside to close (optional feature)
        if let Some(inner_response) = response {
            if inner_response.response.clicked_elsewhere() {
                // Window was clicked outside - could close here if desired
                // self.open = false;
            }
        }
        
        config_changed
    }

    fn render_serial_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Serial Port Settings:");
        
        egui::Grid::new("serial_settings")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Port:");
                ui.text_edit_singleline(&mut self.serial_port);
                ui.end_row();

                ui.label("Baud Rate:");
                ui.text_edit_singleline(&mut self.serial_baudrate);
                ui.end_row();
            });

        ui.add_space(5.0);
        ui.small("Examples: COM3, /dev/ttyUSB0, /dev/ttyACM0");
    }

    fn render_gpsd_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("gpsd Connection Settings:");
        
        egui::Grid::new("gpsd_settings")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Host:");
                ui.text_edit_singleline(&mut self.gpsd_host);
                ui.end_row();

                ui.label("Port:");
                ui.text_edit_singleline(&mut self.gpsd_port);
                ui.end_row();
            });

        ui.add_space(5.0);
        ui.small("Default: localhost:2947");
    }

    #[cfg(windows)]
    fn render_windows_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Windows Location Service Settings:");
        
        egui::Grid::new("windows_settings")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Accuracy (meters):");
                ui.text_edit_singleline(&mut self.windows_accuracy);
                ui.end_row();

                ui.label("Update Interval (seconds):");
                ui.text_edit_singleline(&mut self.windows_interval);
                ui.end_row();
            });

        ui.add_space(5.0);
        ui.small("Lower accuracy values request higher precision (uses more power)");
    }

    fn validate_and_save(&mut self) -> bool {
        match self.source_type {
            SourceType::Serial => {
                if self.serial_port.is_empty() {
                    self.status_message = Some("Error: Serial port cannot be empty".to_string());
                    return false;
                }

                let baudrate = match self.serial_baudrate.parse::<u32>() {
                    Ok(b) => b,
                    Err(_) => {
                        self.status_message = Some("Error: Invalid baud rate".to_string());
                        return false;
                    }
                };

                self.config.update_serial(self.serial_port.clone(), baudrate);
            }
            SourceType::Gpsd => {
                if self.gpsd_host.is_empty() {
                    self.status_message = Some("Error: gpsd host cannot be empty".to_string());
                    return false;
                }

                let port = match self.gpsd_port.parse::<u16>() {
                    Ok(p) => p,
                    Err(_) => {
                        self.status_message = Some("Error: Invalid port number".to_string());
                        return false;
                    }
                };

                self.config.update_gpsd(self.gpsd_host.clone(), port);
            }
            #[cfg(windows)]
            SourceType::Windows => {
                let accuracy = match self.windows_accuracy.parse::<u32>() {
                    Ok(a) => a,
                    Err(_) => {
                        self.status_message = Some("Error: Invalid accuracy value".to_string());
                        return false;
                    }
                };

                let interval = match self.windows_interval.parse::<u64>() {
                    Ok(i) => i,
                    Err(_) => {
                        self.status_message = Some("Error: Invalid interval value".to_string());
                        return false;
                    }
                };

                self.config.update_windows(accuracy, interval);
            }
        }

        // Save to storage
        match self.config.save() {
            Ok(_) => true,
            Err(e) => {
                self.status_message = Some(format!("Error saving: {}", e));
                false
            }
        }
    }

    pub fn get_config(&self) -> &GpsConfig {
        &self.config
    }
}
