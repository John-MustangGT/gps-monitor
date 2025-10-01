// src/display/gui/mod.rs v8
//! GUI display module - Pure egui implementation

pub mod app;
mod panels;
mod satellites;
mod skyplot;
mod settings;
mod waypoint_dialog;

pub use app::{GpsGuiApp, SatelliteSortColumn};
pub use settings::SettingsWindow;
pub use waypoint_dialog::WaypointDialog;
