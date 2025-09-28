// src/gps/data.rs
//! GPS data structures and utilities

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
pub struct GpsData {
    pub timestamp: Option<DateTime<Utc>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub speed: Option<f64>,      // km/h
    pub course: Option<f64>,     // degrees
    pub satellites: Option<u8>,
    pub fix_quality: Option<u8>,
    pub hdop: Option<f64>,
    pub mode: Option<u8>,
    pub accuracy: Option<f64>,   // meters
    pub source: Option<String>,  // GPS, Network, etc.
    pub raw_data: String,
}

impl GpsData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the GPS data represents a valid position fix
    pub fn has_fix(&self) -> bool {
        self.latitude.is_some() && self.longitude.is_some()
    }

    /// Get the age of the GPS data in seconds
    pub fn age_seconds(&self) -> Option<i64> {
        self.timestamp.map(|ts| Utc::now().signed_duration_since(ts).num_seconds())
    }

    /// Check if the GPS data is recent (within 10 seconds)
    pub fn is_recent(&self) -> bool {
        self.age_seconds().map_or(false, |age| age < 10)
    }

    /// Update the timestamp to now
    pub fn update_timestamp(&mut self) {
        self.timestamp = Some(Utc::now());
    }

    /// Set the data source
    pub fn set_source(&mut self, source: &str) {
        self.source = Some(source.to_string());
    }

    /// Get fix type description
    pub fn get_fix_description(&self) -> String {
        if let Some(quality) = self.fix_quality {
            match quality {
                0 => "No fix".to_string(),
                1 => "GPS".to_string(),
                2 => "DGPS".to_string(),
                3 => "PPS".to_string(),
                4 => "RTK".to_string(),
                5 => "Float RTK".to_string(),
                6 => "Estimated".to_string(),
                7 => "Manual".to_string(),
                8 => "Simulation".to_string(),
                _ => format!("Unknown ({})", quality),
            }
        } else if let Some(m) = self.mode {
            match m {
                1 => "No fix".to_string(),
                2 => "2D fix".to_string(),
                3 => "3D fix".to_string(),
                _ => format!("Mode {}", m),
            }
        } else {
            "Unknown".to_string()
        }
    }

    /// Format coordinate for display
    pub fn format_coordinate(coord: Option<f64>) -> String {
        match coord {
            Some(val) => format!("{:>12.6}°", val),
            None => "No fix".to_string(),
        }
    }

    /// Format value with unit for display
    pub fn format_value<T: std::fmt::Display>(value: Option<T>, unit: &str) -> String {
        match value {
            Some(val) => format!("{:>12} {}", val, unit),
            None => "Unknown".to_string(),
        }
    }
}
