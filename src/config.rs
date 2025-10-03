// src/config.rs v2
//! Configuration management with platform-specific storage

use crate::error::{Result, GpsError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsConfig {
    pub source_type: String,  // "serial", "gpsd", "windows"
    pub serial_port: Option<String>,
    pub serial_baudrate: Option<u32>,
    pub gpsd_host: Option<String>,
    pub gpsd_port: Option<u16>,
    pub windows_accuracy: Option<u32>,
    pub windows_interval: Option<u64>,
}

impl Default for GpsConfig {
    fn default() -> Self {
        Self::platform_default()
    }
}

impl GpsConfig {
    /// Get platform-specific default configuration
    pub fn platform_default() -> Self {
        #[cfg(windows)]
        {
            Self {
                source_type: "windows".to_string(),
                serial_port: None,
                serial_baudrate: Some(9600),
                gpsd_host: Some("localhost".to_string()),
                gpsd_port: Some(2947),
                windows_accuracy: Some(10),
                windows_interval: Some(1),
            }
        }

        #[cfg(not(windows))]
        {
            Self {
                source_type: "gpsd".to_string(),
                serial_port: None,
                serial_baudrate: Some(9600),
                gpsd_host: Some("localhost".to_string()),
                gpsd_port: Some(2947),
                windows_accuracy: Some(10),
                windows_interval: Some(1),
            }
        }
    }

    /// Load configuration from storage
    pub fn load() -> Result<Self> {
        #[cfg(windows)]
        {
            Self::load_from_registry()
        }

        #[cfg(not(windows))]
        {
            Self::load_from_file()
        }
    }

    /// Save configuration to storage
    pub fn save(&self) -> Result<()> {
        #[cfg(windows)]
        {
            self.save_to_registry()
        }

        #[cfg(not(windows))]
        {
            self.save_to_file()
        }
    }

    /// Load from Windows Registry
    #[cfg(windows)]
    fn load_from_registry() -> Result<Self> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = r"Software\GpsMonitor";
        
        match hkcu.open_subkey(key_path) {
            Ok(key) => {
                let source_type: String = key.get_value("SourceType")
                    .unwrap_or_else(|_| "windows".to_string());
                
                // Convert u32 to u16 for gpsd_port
                let gpsd_port_u32: Option<u32> = key.get_value("GpsdPort").ok();
                let gpsd_port = gpsd_port_u32.map(|p| p as u16);
                
                // Convert u32 to u64 for windows_interval
                let windows_interval_u32: Option<u32> = key.get_value("WindowsInterval").ok();
                let windows_interval = windows_interval_u32.map(|i| i as u64);
                
                let config = Self {
                    source_type,
                    serial_port: key.get_value("SerialPort").ok(),
                    serial_baudrate: key.get_value("SerialBaudrate").ok(),
                    gpsd_host: key.get_value("GpsdHost").ok(),
                    gpsd_port,
                    windows_accuracy: key.get_value("WindowsAccuracy").ok(),
                    windows_interval,
                };
                
                Ok(config)
            }
            Err(_) => {
                // Registry key doesn't exist, return default
                Ok(Self::platform_default())
            }
        }
    }

    /// Save to Windows Registry
    #[cfg(windows)]
    fn save_to_registry(&self) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = r"Software\GpsMonitor";
        
        let (key, _) = hkcu.create_subkey(key_path)
            .map_err(|e| GpsError::Other(format!("Failed to create registry key: {}", e)))?;
        
        key.set_value("SourceType", &self.source_type)
            .map_err(|e| GpsError::Other(format!("Failed to save SourceType: {}", e)))?;
        
        if let Some(ref port) = self.serial_port {
            key.set_value("SerialPort", port)
                .map_err(|e| GpsError::Other(format!("Failed to save SerialPort: {}", e)))?;
        }
        
        if let Some(baudrate) = self.serial_baudrate {
            key.set_value("SerialBaudrate", &baudrate)
                .map_err(|e| GpsError::Other(format!("Failed to save SerialBaudrate: {}", e)))?;
        }
        
        if let Some(ref host) = self.gpsd_host {
            key.set_value("GpsdHost", host)
                .map_err(|e| GpsError::Other(format!("Failed to save GpsdHost: {}", e)))?;
        }
        
        // Convert u16 to u32 for registry storage
        if let Some(port) = self.gpsd_port {
            let port_u32 = port as u32;
            key.set_value("GpsdPort", &port_u32)
                .map_err(|e| GpsError::Other(format!("Failed to save GpsdPort: {}", e)))?;
        }
        
        if let Some(accuracy) = self.windows_accuracy {
            key.set_value("WindowsAccuracy", &accuracy)
                .map_err(|e| GpsError::Other(format!("Failed to save WindowsAccuracy: {}", e)))?;
        }
        
        // Convert u64 to u32 for registry storage
        if let Some(interval) = self.windows_interval {
            let interval_u32 = interval as u32;
            key.set_value("WindowsInterval", &interval_u32)
                .map_err(|e| GpsError::Other(format!("Failed to save WindowsInterval: {}", e)))?;
        }
        
        Ok(())
    }

    /// Load from config file on Unix systems
    #[cfg(not(windows))]
    fn load_from_file() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::platform_default());
        }
        
        let contents = std::fs::read_to_string(&config_path)
            .map_err(|e| GpsError::Other(format!("Failed to read config file: {}", e)))?;
        
        let config: Self = serde_json::from_str(&contents)
            .map_err(|e| GpsError::Other(format!("Failed to parse config file: {}", e)))?;
        
        Ok(config)
    }

    /// Save to config file on Unix systems
    #[cfg(not(windows))]
    fn save_to_file(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| GpsError::Other(format!("Failed to create config directory: {}", e)))?;
        }
        
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| GpsError::Other(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(&config_path, contents)
            .map_err(|e| GpsError::Other(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }

    /// Get config file path for Unix systems
    #[cfg(not(windows))]
    fn get_config_path() -> Result<std::path::PathBuf> {
        use std::path::PathBuf;
        
        let home = std::env::var("HOME")
            .map_err(|_| GpsError::Other("HOME environment variable not set".to_string()))?;
        
        Ok(PathBuf::from(home).join(".config").join("gps-monitor").join("config.json"))
    }

    /// Update configuration with new source settings
    pub fn update_source(&mut self, source_type: &str) {
        self.source_type = source_type.to_string();
    }

    /// Update serial port settings
    pub fn update_serial(&mut self, port: String, baudrate: u32) {
        self.source_type = "serial".to_string();
        self.serial_port = Some(port);
        self.serial_baudrate = Some(baudrate);
    }

    /// Update gpsd settings
    pub fn update_gpsd(&mut self, host: String, port: u16) {
        self.source_type = "gpsd".to_string();
        self.gpsd_host = Some(host);
        self.gpsd_port = Some(port);
    }

    /// Update Windows location settings
    pub fn update_windows(&mut self, accuracy: u32, interval: u64) {
        self.source_type = "windows".to_string();
        self.windows_accuracy = Some(accuracy);
        self.windows_interval = Some(interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GpsConfig::default();
        
        #[cfg(windows)]
        assert_eq!(config.source_type, "windows");
        
        #[cfg(not(windows))]
        assert_eq!(config.source_type, "gpsd");
    }

    #[test]
    fn test_update_source() {
        let mut config = GpsConfig::default();
        config.update_source("serial");
        assert_eq!(config.source_type, "serial");
    }

    #[test]
    fn test_update_serial() {
        let mut config = GpsConfig::default();
        config.update_serial("/dev/ttyUSB0".to_string(), 115200);
        assert_eq!(config.source_type, "serial");
        assert_eq!(config.serial_port, Some("/dev/ttyUSB0".to_string()));
        assert_eq!(config.serial_baudrate, Some(115200));
    }
}