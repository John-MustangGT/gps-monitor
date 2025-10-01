// src/display/gui/mod.rs v9
//! GUI display module - Pure egui implementation

pub mod app;
mod panels;
mod satellites;
mod skyplot;
mod settings;
mod waypoint_dialog;
mod track_recorder;

pub use app::{GpsGuiApp, SatelliteSortColumn};
pub use settings::SettingsWindow;
pub use waypoint_dialog::WaypointDialog;
