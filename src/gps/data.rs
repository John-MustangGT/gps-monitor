// src/gps/data.rs
//! GPS data structures and utilities

use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SatelliteInfo {
    pub prn: u8,           // Satellite PRN/ID number
    pub elevation: Option<f32>,  // Elevation angle in degrees
    pub azimuth: Option<f32>,    // Azimuth angle in degrees
    pub snr: Option<f32>,        // Signal-to-noise ratio in dB
    pub used: bool,              // Whether satellite is used in fix
    pub constellation: String,   // GPS, GLONASS, GALILEO, BEIDOU, etc.
}

impl SatelliteInfo {
    pub fn new(prn: u8) -> Self {
        Self {
            prn,
            elevation: None,
            azimuth: None,
            snr: None,
            used: false,
            constellation: Self::determine_constellation(prn),
        }
    }

    fn determine_constellation(prn: u8) -> String {
        match prn {
            1..=32 => "GPS".to_string(),
            33..=64 => "SBAS".to_string(),
            65..=96 => "GLONASS".to_string(),
            120..=158 => "BEIDOU".to_string(),
            159..=163 => "BEIDOU".to_string(),
            193..=197 => "QZSS".to_string(),
            211..=246 => "GALILEO".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    pub fn signal_strength_description(&self) -> String {
        match self.snr {
            Some(snr) if snr >= 40.0 => "Excellent".to_string(),
            Some(snr) if snr >= 35.0 => "Good".to_string(),
            Some(snr) if snr >= 25.0 => "Fair".to_string(),
            Some(snr) if snr >= 15.0 => "Poor".to_string(),
            Some(_) => "Very Poor".to_string(),
            None => "Unknown".to_string(),
        }
    }
}

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
    pub raw_history: Vec<String>, // Recent NMEA sentences
    pub satellites_info: Vec<SatelliteInfo>, // Detailed satellite information
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

    /// Add a raw NMEA sentence to history (keep last 5)
    pub fn add_raw_sentence(&mut self, sentence: &str) {
        self.raw_data = sentence.to_string();
        self.raw_history.push(sentence.to_string());
        
        // Keep only the last 5 sentences
        if self.raw_history.len() > 5 {
            self.raw_history.remove(0);
        }
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

    /// Get count of satellites being used in the fix
    pub fn satellites_used(&self) -> usize {
        self.satellites_info.iter().filter(|sat| sat.used).count()
    }

    /// Get satellites grouped by constellation
    pub fn satellites_by_constellation(&self) -> HashMap<String, Vec<&SatelliteInfo>> {
        let mut grouped = HashMap::new();
        for sat in &self.satellites_info {
            grouped.entry(sat.constellation.clone()).or_insert_with(Vec::new).push(sat);
        }
        grouped
    }
}
