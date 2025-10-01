// src/display/mod.rs v4
//! Display modules - Pure egui implementation

#[cfg(feature = "gui")]
pub mod gui;

// Always include terminal module for non-GUI builds
pub mod terminal;

#[cfg(not(feature = "gui"))]
pub mod gui {
    // Stub for when GUI is not enabled
}
