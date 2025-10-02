// src/lib.rs v6
//! GPS Monitor Library
//! 
//! A cross-platform GPS monitoring library that supports multiple GPS sources
//! and display modes.

pub mod gps;
pub mod display;
pub mod monitor;
pub mod error;
pub mod config;
pub mod waypoint;
pub mod map;

// Re-export main types for convenience
pub use gps::data::GpsData;
pub use monitor::{GpsMonitor, GpsSource};
pub use error::{Result, GpsError};
pub use config::GpsConfig;
pub use waypoint::{Waypoint, WaypointExporter, WaypointFormat, Track, TrackPoint};
pub use map::{TileCache, CacheStats};

#[cfg(feature = "gui")]
pub use display::gui::GpsGuiApp;
