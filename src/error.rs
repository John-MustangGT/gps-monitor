// src/error.rs
//! Error types for the GPS monitor

use std::fmt;

pub type Result<T> = std::result::Result<T, GpsError>;

#[derive(Debug)]
pub enum GpsError {
    Io(std::io::Error),
    Serial(tokio_serial::Error),
    Json(serde_json::Error),
    Connection(String),
    Parse(String),
    #[cfg(windows)]
    Windows(windows::core::Error),
    #[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
    Gui(eframe::Error),
    Other(String),
}

impl fmt::Display for GpsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpsError::Io(e) => write!(f, "IO error: {}", e),
            GpsError::Serial(e) => write!(f, "Serial error: {}", e),
            GpsError::Json(e) => write!(f, "JSON error: {}", e),
            GpsError::Connection(msg) => write!(f, "Connection error: {}", msg),
            GpsError::Parse(msg) => write!(f, "Parse error: {}", msg),
            #[cfg(windows)]
            GpsError::Windows(e) => write!(f, "Windows error: {}", e),
            #[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
            GpsError::Gui(e) => write!(f, "GUI error: {}", e),
            GpsError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for GpsError {}

impl From<std::io::Error> for GpsError {
    fn from(error: std::io::Error) -> Self {
        GpsError::Io(error)
    }
}

impl From<tokio_serial::Error> for GpsError {
    fn from(error: tokio_serial::Error) -> Self {
        GpsError::Serial(error)
    }
}

impl From<serde_json::Error> for GpsError {
    fn from(error: serde_json::Error) -> Self {
        GpsError::Json(error)
    }
}

#[cfg(windows)]
impl From<windows::core::Error> for GpsError {
    fn from(error: windows::core::Error) -> Self {
        GpsError::Windows(error)
    }
}

#[cfg(all(unix, not(target_os = "macos"), feature = "gui"))]
impl From<eframe::Error> for GpsError {
    fn from(error: eframe::Error) -> Self {
        GpsError::Gui(error)
    }
}

impl From<anyhow::Error> for GpsError {
    fn from(error: anyhow::Error) -> Self {
        GpsError::Other(error.to_string())
    }
}
