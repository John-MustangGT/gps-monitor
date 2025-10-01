// src/main.rs v3
//! GPS Monitor - Cross-platform GPS monitoring tool with egui

use gps_monitor::{config::GpsConfig, *};

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("Error: This application requires the 'gui' feature.");
    eprintln!("Build with: cargo build --features gui");
    std::process::exit(1);
}

#[cfg(feature = "gui")]
fn main() -> Result<()> {
    // Load configuration
    let config = GpsConfig::load().unwrap_or_default();
    
    println!("Starting GPS Monitor...");
    println!("Using {} source", config.source_type);
    
    // Create and run the egui application
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("GPS Monitor")
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GPS Monitor",
        options,
        Box::new(|cc| {
            // Set visual style
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            
            Ok(Box::new(display::gui::GpsGuiApp::new_from_config(config)))
        }),
    )
    .map_err(|e| error::GpsError::Other(format!("GUI error: {}", e)))?;

    Ok(())
}
