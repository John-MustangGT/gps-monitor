// src/display/gui/mod.rs v3
//! GUI display module - Pure egui implementation

mod app;
mod panels;
mod satellites;
mod skyplot;
mod settings;

pub use app::{GpsGuiApp, SatelliteSortColumn};
pub use settings::SettingsWindow;
