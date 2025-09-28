// src/lib.rs
//! GPS Monitor Library
//! 
//! A cross-platform GPS monitoring library that supports multiple GPS sources
//! and display modes.

pub mod gps;
pub mod display;
pub mod monitor;
pub mod error;

// Re-export main types for convenience
pub use gps::data::GpsData;
pub use monitor::{GpsMonitor, GpsSource};
pub use error::{Result, GpsError};

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub use display::gui::GpsGuiApp;
