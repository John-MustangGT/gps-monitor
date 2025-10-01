// src/monitor.rs v2
/// Main GPS monitor coordination

use crate::{
    display::terminal::TerminalDisplay,
    error::{Result, GpsError},
    gps::{data::GpsData, gpsd, nmea},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_serial::SerialPortBuilderExt;

#[cfg(windows)]
use crate::gps::windows;

/// GPS data source configuration
#[derive(Debug, Clone)]
pub enum GpsSource {
    Serial { port: String, baudrate: u32 },
    Gpsd { host: String, port: u16 },
    #[cfg(windows)]
    Windows { accuracy: u32, interval: u64 },
}

/// Main GPS monitor that coordinates data collection and display
pub struct GpsMonitor {
    data: Arc<RwLock<GpsData>>,
    running: Arc<AtomicBool>,
}

impl GpsMonitor {
    /// Create a new GPS monitor
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(GpsData::new())),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Create a new GPS monitor with shared data and running flag
    pub fn new_with_shared(
        data: Arc<RwLock<GpsData>>,
        running: Arc<AtomicBool>,
    ) -> Self {
        Self {
            data,
            running,
        }
    }

    /// Clone the monitor (shares data and running flag)
    pub fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            running: Arc::clone(&self.running),
        }
    }

    /// Start monitoring GPS data from the specified source
    pub async fn start(&self, source: GpsSource) -> Result<()> {
        match source {
            GpsSource::Serial { port, baudrate } => {
                self.connect_serial(&port, baudrate).await?;
            }
            GpsSource::Gpsd { host, port } => {
                self.connect_gpsd(&host, port).await?;
            }
            #[cfg(windows)]
            GpsSource::Windows { accuracy, interval } => {
                self.connect_windows_location(accuracy, interval).await?;
            }
        }
        Ok(())
    }

    /// Start the display (terminal only for now)
    pub async fn run_display(&self) -> Result<()> {
        let terminal_display = TerminalDisplay::new();
        terminal_display.run(Arc::clone(&self.data), Arc::clone(&self.running)).await
    }

    /// Connect to a GPS device via serial port
    async fn connect_serial(&self, port: &str, baudrate: u32) -> Result<()> {
        println!("Connecting to GPS on {} at {} baud...", port, baudrate);

        let serial = tokio_serial::new(port, baudrate)
            .timeout(Duration::from_millis(1000))
            .open_native_async()
            .map_err(|e| GpsError::Connection(format!("Failed to open serial port {}: {}", port, e)))?;

        println!("Connected successfully!");

        let data = Arc::clone(&self.data);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut reader = BufReader::new(serial);
            let mut line = String::new();

            while running.load(Ordering::Relaxed) {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let line = line.trim();
                        if !line.is_empty() {
                            let mut data_guard = data.write().unwrap();
                            data_guard.update_timestamp();
                            data_guard.add_raw_sentence(line);
                            data_guard.set_source("Serial GPS");
                            nmea::parse_nmea_sentence(&mut data_guard, line);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from serial port: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect to gpsd daemon
    async fn connect_gpsd(&self, host: &str, port: u16) -> Result<()> {
        println!("Connecting to gpsd at {}:{}...", host, port);

        let mut reader = gpsd::connect_gpsd(host, port).await?;
        println!("Connected successfully!");

        let data = Arc::clone(&self.data);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut line = String::new();

            while running.load(Ordering::Relaxed) {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let line = line.trim();
                        if !line.is_empty() {
                            let mut data_guard = data.write().unwrap();
                            data_guard.update_timestamp();
                            data_guard.add_raw_sentence(line);
                            data_guard.set_source("gpsd");
                            
                            if let Err(e) = gpsd::parse_gpsd_json(&mut data_guard, line) {
                                eprintln!("Error parsing gpsd JSON: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from gpsd: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect to Windows Location Services
    #[cfg(windows)]
    #[allow(dead_code)]
    async fn connect_windows_location(&self, accuracy: u32, interval: u64) -> Result<()> {
        println!("Connecting to Windows Location Service...");

        // Request access and create geolocator
        windows::request_location_access().await?;
        let geolocator = windows::create_geolocator(accuracy)?;

        println!("Windows Location Service initialized successfully!");

        // Start monitoring
        windows::run_location_monitoring(
            geolocator,
            Arc::clone(&self.data),
            Arc::clone(&self.running),
            interval,
        ).await;

        Ok(())
    }

    #[cfg(not(windows))]
    #[allow(dead_code)]
    async fn connect_windows_location(&self, _accuracy: u32, _interval: u64) -> Result<()> {
        Err(GpsError::Other("Windows Location Service is only available on Windows".to_string()))
    }

    /// Stop the monitor
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Check if the monitor is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get a clone of the current GPS data
    pub fn get_data(&self) -> GpsData {
        self.data.read().unwrap().clone()
    }
}

impl Default for GpsMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// List available serial ports
pub async fn list_serial_ports() -> Result<()> {
    let ports = tokio_serial::available_ports()
        .map_err(|e| GpsError::Other(format!("Failed to list serial ports: {}", e)))?;

    if ports.is_empty() {
        println!("No serial ports found.");
    } else {
        println!("Available serial ports:");
        for port in ports {
            println!("  {} - {:?}", port.port_name, port.port_type);
        }
    }

    Ok(())
}
