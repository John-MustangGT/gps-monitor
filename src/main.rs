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
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let start_fullscreen = args.iter().any(|arg| arg == "--fullscreen" || arg == "-f");

    // Load configuration
    let config = GpsConfig::load().unwrap_or_default();

    println!("Starting GPS Monitor...");
    println!("Using {} source", config.source_type);
    if start_fullscreen {
        println!("Starting in fullscreen mode");
    }

    // Create and run the egui application
    let mut viewport_builder = eframe::egui::ViewportBuilder::default()
        .with_inner_size([1024.0, 768.0])
        .with_title("GPS Monitor")
        .with_min_inner_size([800.0, 600.0]);

    if start_fullscreen {
        viewport_builder = viewport_builder.with_fullscreen(true);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
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
