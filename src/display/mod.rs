// src/display/mod.rs v3
//! Display modules - Pure egui implementation

#[cfg(feature = "gui")]
pub mod gui;

#[cfg(not(feature = "gui"))]
pub mod gui {
    // Stub for when GUI is not enabled
}
