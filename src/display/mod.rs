// src/display/mod.rs
//! Display modules for different interfaces

pub mod terminal;

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub mod gui;

use crate::gps::GpsData;
use std::sync::{Arc, RwLock};

/// Trait for different display implementations
pub trait GpsDisplay {
    type Error;
    
    /// Start the display loop
    fn start_display(
        &self,
        data: Arc<RwLock<GpsData>>,
        running: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<(), Self::Error>;
}

/// Check if GUI should be used based on environment
#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub fn should_use_gui() -> bool {
    std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
}

#[cfg(not(all(unix, not(target_os = "macos"), feature = "gui")))]
pub fn should_use_gui() -> bool {
    false
}
