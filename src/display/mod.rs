// src/display/mod.rs v2
//! Display modules for different interfaces

pub mod terminal;

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub mod gui;

#[cfg(not(all(unix, not(target_os = "macos"), feature = "gui")))]
pub mod gui {
    use crate::{gps::GpsData, error::{Result, GpsError}};
    use std::sync::{Arc, RwLock, atomic::AtomicBool};
    
    pub struct GuiDisplay;
    
    impl GuiDisplay {
        pub fn new() -> Self {
            Self
        }
        
        pub async fn run(
            &self,
            _data: Arc<RwLock<GpsData>>,
            _running: Arc<AtomicBool>,
        ) -> Result<()> {
            Err(GpsError::Other("GUI support not compiled in".to_string()))
        }
    }
}

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
