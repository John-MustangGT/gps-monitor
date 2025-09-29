// src/display/gui/mod.rs v1
//! GUI display module - Main orchestration

mod app;
mod panels;
mod satellites;
mod skyplot;

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub use app::GpsGuiApp;

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
use {
    crate::{
        gps::GpsData,
        error::{Result, GpsError},
    },
    std::sync::{
        atomic::AtomicBool,
        mpsc, Arc, RwLock,
    },
};

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
pub struct GuiDisplay;

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl GuiDisplay {
    pub fn new() -> Self {
        Self
    }

    /// Start the GUI display
    pub async fn run(
        &self,
        data: Arc<RwLock<GpsData>>,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        let (tx, _rx) = mpsc::channel();
        let data_clone = Arc::clone(&data);
        let running_clone = Arc::clone(&running);

        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_title("GPS Monitor"),
            ..Default::default()
        };

        let app = GpsGuiApp::new(data_clone, running_clone, tx);
        
        match eframe::run_native("GPS Monitor", options, Box::new(|_cc| Ok(Box::new(app)))) {
            Ok(_) => Ok(()),
            Err(e) => Err(GpsError::Other(format!("GUI error: {}", e))),
        }
    }
}

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl Default for GuiDisplay {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementations for non-GUI builds
#[cfg(not(all(unix, not(target_os = "macos"), feature = "gui")))]
pub struct GuiDisplay;

#[cfg(not(all(unix, not(target_os = "macos"), feature = "gui")))]
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
