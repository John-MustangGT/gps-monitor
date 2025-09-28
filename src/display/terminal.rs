// src/display/terminal.rs
//! Terminal-based display implementation

use crate::{
    gps::GpsData,
    error::{Result, GpsError},
};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType, DisableLineWrap, EnableLineWrap},
};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};
use tokio::time::sleep;

pub struct TerminalDisplay;

impl TerminalDisplay {
    pub fn new() -> Self {
        Self
    }

    /// Start the terminal display loop
    pub async fn run(
        &self,
        data: Arc<RwLock<GpsData>>,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, Hide, DisableLineWrap)
            .map_err(|e| GpsError::Io(e))?;

        // Set up Ctrl+C handler
        let running_clone = Arc::clone(&running);
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            running_clone.store(false, Ordering::Relaxed);
        });

        while running.load(Ordering::Relaxed) {
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))
                .map_err(|e| GpsError::Io(e))?;

            let gps_data = data.read().unwrap().clone();
            self.render_display(&mut stdout, &gps_data)?;

            stdout.flush().map_err(|e| GpsError::Io(e))?;
            sleep(Duration::from_secs(1)).await;
        }

        execute!(stdout, Show, EnableLineWrap)
            .map_err(|e| GpsError::Io(e))?;
        println!("\nShutting down...");
        Ok(())
    }

    /// Render the GPS data to the terminal
    fn render_display(&self, stdout: &mut impl Write, data: &GpsData) -> Result<()> {
        // Header
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("=".repeat(60)),
            Print("\n"),
            Print("GPS Monitor - Cross Platform GPS Display (Rust)"),
            Print("\n"),
            Print("=".repeat(60)),
            Print("\n"),
            ResetColor
        ).map_err(|e| GpsError::Io(e))?;

        // Timestamp and source
        let timestamp_str = match data.timestamp {
            Some(ts) => ts.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            None => "No data received".to_string(),
        };
        let source_str = data.source.as_deref().unwrap_or("Unknown");
        execute!(
            stdout,
            Print(format!("Last Update: {} ({})\n\n", timestamp_str, source_str))
        ).map_err(|e| GpsError::Io(e))?;

        // Position section
        self.render_position_section(stdout, data)?;

        // Movement section
        self.render_movement_section(stdout, data)?;

        // Quality section (for GPS sources)
        if data.satellites.is_some() || data.hdop.is_some() || data.fix_quality.is_some() {
            self.render_quality_section(stdout, data)?;
        }

        // Raw data section
        self.render_raw_data_section(stdout, data)?;

        // Footer
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("=".repeat(60)),
            Print("\n"),
            Print("Press Ctrl+C to exit"),
            Print("\n"),
            ResetColor
        ).map_err(|e| GpsError::Io(e))?;

        Ok(())
    }

    fn render_position_section(&self, stdout: &mut impl Write, data: &GpsData) -> Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("POSITION:\n"),
            ResetColor
        ).map_err(|e| GpsError::Io(e))?;

        execute!(
            stdout,
            Print(format!("  Latitude:  {}\n", GpsData::format_coordinate(data.latitude)))
        ).map_err(|e| GpsError::Io(e))?;

        execute!(
            stdout,
            Print(format!("  Longitude: {}\n", GpsData::format_coordinate(data.longitude)))
        ).map_err(|e| GpsError::Io(e))?;

        execute!(
            stdout,
            Print(format!("  Altitude:  {}\n", GpsData::format_value(data.altitude, "m")))
        ).map_err(|e| GpsError::Io(e))?;

        if let Some(acc) = data.accuracy {
            execute!(
                stdout,
                Print(format!("  Accuracy:  {:>12.1} m\n", acc))
            ).map_err(|e| GpsError::Io(e))?;
        }

        execute!(stdout, Print("\n")).map_err(|e| GpsError::Io(e))?;
        Ok(())
    }

    fn render_movement_section(&self, stdout: &mut impl Write, data: &GpsData) -> Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("MOVEMENT:\n"),
            ResetColor
        ).map_err(|e| GpsError::Io(e))?;

        execute!(
            stdout,
            Print(format!("  Speed:     {}\n", GpsData::format_value(data.speed, "km/h")))
        ).map_err(|e| GpsError::Io(e))?;

        execute!(
            stdout,
            Print(format!("  Course:    {}\n\n", GpsData::format_value(data.course, "°")))
        ).map_err(|e| GpsError::Io(e))?;

        Ok(())
    }

    fn render_quality_section(&self, stdout: &mut impl Write, data: &GpsData) -> Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Magenta),
            Print("QUALITY:\n"),
            ResetColor
        ).map_err(|e| GpsError::Io(e))?;

        execute!(
            stdout,
            Print(format!("  Satellites: {}\n", GpsData::format_value(data.satellites, "")))
        ).map_err(|e| GpsError::Io(e))?;

        execute!(
            stdout,
            Print(format!("  HDOP:       {}\n", GpsData::format_value(data.hdop, "")))
        ).map_err(|e| GpsError::Io(e))?;

        let fix_type = data.get_fix_description();
        execute!(
            stdout,
            Print(format!("  Fix Type:   {:>11}\n\n", fix_type))
        ).map_err(|e| GpsError::Io(e))?;

        Ok(())
    }

    fn render_raw_data_section(&self, stdout: &mut impl Write, data: &GpsData) -> Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Blue),
            Print("RAW DATA:\n"),
            ResetColor
        ).map_err(|e| GpsError::Io(e))?;

        let raw_display = if data.raw_data.is_empty() {
            "No data"
        } else {
            &data.raw_data
        };

        execute!(
            stdout,
            Print(format!("  {}\n\n", raw_display))
        ).map_err(|e| GpsError::Io(e))?;

        Ok(())
    }
}

impl Default for TerminalDisplay {
    fn default() -> Self {
        Self::new()
    }
}
