// src/main.rs
//! GPS Monitor - Cross-platform GPS monitoring tool

use clap::{Parser, Subcommand};
use gps_monitor::{monitor::*, *};

#[derive(Parser)]
#[command(name = "gps-monitor")]
#[command(about = "Cross-platform GPS monitoring tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Force terminal mode even if GUI is available
    #[arg(long, global = true)]
    terminal: bool,
    
    /// Force GUI mode (requires GUI feature and X11)
    #[arg(long, global = true)]
    gui: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List available serial ports
    ListPorts,
    /// Connect to GPS via serial port
    Serial {
        /// Serial port path (e.g., COM3, /dev/ttyUSB0)
        port: String,
        /// Baud rate
        #[arg(short, long, default_value = "9600")]
        baudrate: u32,
    },
    /// Connect to gpsd daemon
    Gpsd {
        /// gpsd host
        #[arg(short = 'H', long, default_value = "localhost")]
        host: String,
        /// gpsd port
        #[arg(short, long, default_value = "2947")]
        port: u16,
    },
    #[cfg(windows)]
    /// Use Windows Location Service
    Windows {
        /// Desired accuracy in meters
        #[arg(short, long, default_value = "10")]
        accuracy: u32,
        /// Update interval in seconds
        #[arg(short, long, default_value = "1")]
        interval: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check for GUI requirements
    #[cfg(not(all(unix, not(target_os = "macos"), feature = "gui")))]
    if cli.gui {
        eprintln!("Error: GUI support not compiled in. Build with --features gui");
        std::process::exit(1);
    }

    match cli.command {
        Commands::ListPorts => {
            list_serial_ports().await?;
        }
        Commands::Serial { port, baudrate } => {
            let monitor = GpsMonitor::new();
            let source = GpsSource::Serial { port, baudrate };
            
            monitor.start(source).await?;
            monitor.run_display(cli.terminal, cli.gui).await?;
        }
        Commands::Gpsd { host, port } => {
            let monitor = GpsMonitor::new();
            let source = GpsSource::Gpsd { host, port };
            
            monitor.start(source).await?;
            monitor.run_display(cli.terminal, cli.gui).await?;
        }
        #[cfg(windows)]
        Commands::Windows { accuracy, interval } => {
            let monitor = GpsMonitor::new();
            let source = GpsSource::Windows { accuracy, interval };
            
            monitor.start(source).await?;
            monitor.run_display(cli.terminal, cli.gui).await?;
        }
    }

    Ok(())
}
