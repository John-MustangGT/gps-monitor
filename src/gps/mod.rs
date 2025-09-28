// src/gps/mod.rs
//! GPS data handling and parsing

pub mod data;
pub mod nmea;
pub mod gpsd;

#[cfg(windows)]
pub mod windows;

pub use data::GpsData;
