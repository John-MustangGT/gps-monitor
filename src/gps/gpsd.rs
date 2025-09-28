// src/gps/gpsd.rs
//! GPSD client implementation

use super::data::{GpsData, SatelliteInfo};
use crate::error::{Result, GpsError};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[derive(Debug, Deserialize)]
struct GpsdMessage {
    class: String,
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

/// Connect to a gpsd daemon and return a stream reader
pub async fn connect_gpsd(host: &str, port: u16) -> Result<BufReader<TcpStream>> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))
        .await
        .map_err(|e| GpsError::Connection(format!("Failed to connect to gpsd at {}:{}: {}", host, port, e)))?;

    // Send WATCH command to start receiving JSON data
    let watch_cmd = format!("?WATCH={{\"enable\":true,\"json\":true}}\n");
    stream
        .write_all(watch_cmd.as_bytes())
        .await
        .map_err(|e| GpsError::Connection(format!("Failed to send WATCH command: {}", e)))?;

    Ok(BufReader::new(stream))
}

/// Parse a single line of gpsd JSON data
pub fn parse_gpsd_json(data: &mut GpsData, line: &str) -> Result<()> {
    let msg: GpsdMessage = serde_json::from_str(line)
        .map_err(|e| GpsError::Parse(format!("Failed to parse gpsd JSON: {}", e)))?;

    match msg.class.as_str() {
        "TPV" => parse_tpv_message(data, &msg.data),
        "SKY" => parse_sky_message(data, &msg.data),
        "VERSION" => parse_version_message(&msg.data),
        "DEVICES" => parse_devices_message(&msg.data),
        _ => {
            // Ignore unknown message types
        }
    }

    Ok(())
}

/// Parse TPV (Time Position Velocity) message
fn parse_tpv_message(data: &mut GpsData, msg_data: &HashMap<String, serde_json::Value>) {
    if let Some(lat) = msg_data.get("lat").and_then(|v| v.as_f64()) {
        data.latitude = Some(lat);
    }
    
    if let Some(lon) = msg_data.get("lon").and_then(|v| v.as_f64()) {
        data.longitude = Some(lon);
    }
    
    if let Some(alt) = msg_data.get("alt").and_then(|v| v.as_f64()) {
        data.altitude = Some(alt);
    }
    
    if let Some(speed) = msg_data.get("speed").and_then(|v| v.as_f64()) {
        data.speed = Some(speed * 3.6); // Convert m/s to km/h
    }
    
    if let Some(track) = msg_data.get("track").and_then(|v| v.as_f64()) {
        data.course = Some(track);
    }
    
    if let Some(mode) = msg_data.get("mode").and_then(|v| v.as_u64()) {
        data.mode = Some(mode as u8);
    }
}

/// Parse SKY (satellite data) message
fn parse_sky_message(data: &mut GpsData, msg_data: &HashMap<String, serde_json::Value>) {
    if let Some(satellites) = msg_data.get("satellites").and_then(|v| v.as_array()) {
        data.satellites_info.clear(); // Clear existing satellite data
        
        for sat_value in satellites {
            if let Some(sat_obj) = sat_value.as_object() {
                if let Some(prn) = sat_obj.get("PRN").and_then(|v| v.as_u64()) {
                    let mut sat_info = SatelliteInfo::new(prn as u8);
                    
                    // Elevation
                    if let Some(el) = sat_obj.get("el").and_then(|v| v.as_f64()) {
                        sat_info.elevation = Some(el as f32);
                    }
                    
                    // Azimuth
                    if let Some(az) = sat_obj.get("az").and_then(|v| v.as_f64()) {
                        sat_info.azimuth = Some(az as f32);
                    }
                    
                    // Signal strength
                    if let Some(ss) = sat_obj.get("ss").and_then(|v| v.as_f64()) {
                        sat_info.snr = Some(ss as f32);
                    }
                    
                    // Used in fix
                    if let Some(used) = sat_obj.get("used").and_then(|v| v.as_bool()) {
                        sat_info.used = used;
                    }
                    
                    data.satellites_info.push(sat_info);
                }
            }
        }
        
        // Update satellite count
        data.satellites = Some(data.satellites_info.len() as u8);
    }
    
    if let Some(hdop) = msg_data.get("hdop").and_then(|v| v.as_f64()) {
        data.hdop = Some(hdop);
    }
}

/// Parse VERSION message (informational)
fn parse_version_message(msg_data: &HashMap<String, serde_json::Value>) {
    if let Some(version) = msg_data.get("release").and_then(|v| v.as_str()) {
        println!("Connected to gpsd version: {}", version);
    }
}

/// Parse DEVICES message (informational)
fn parse_devices_message(msg_data: &HashMap<String, serde_json::Value>) {
    if let Some(devices) = msg_data.get("devices").and_then(|v| v.as_array()) {
        println!("gpsd managing {} device(s)", devices.len());
        for device in devices {
            if let Some(path) = device.get("path").and_then(|v| v.as_str()) {
                println!("  Device: {}", path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tpv_parsing() {
        let mut data = GpsData::new();
        let json = r#"{"class":"TPV","device":"/dev/ttyUSB0","mode":3,"time":"2023-01-01T12:00:00.000Z","ept":0.005,"lat":48.117,"lon":11.517,"alt":545.4,"epx":15.319,"epy":17.054,"epv":124.484,"track":10.3797,"speed":0.091,"climb":10.7,"eps":34.11,"epc":248.97}"#;
        
        parse_gpsd_json(&mut data, json).unwrap();
        
        assert_eq!(data.latitude, Some(48.117));
        assert_eq!(data.longitude, Some(11.517));
        assert_eq!(data.altitude, Some(545.4));
        assert_eq!(data.mode, Some(3));
        assert!((data.speed.unwrap() - 0.3276).abs() < 0.001); // 0.091 m/s * 3.6 = 0.3276 km/h
        assert_eq!(data.course, Some(10.3797));
    }

    #[test]
    fn test_sky_parsing() {
        let mut data = GpsData::new();
        let json = r#"{"class":"SKY","device":"/dev/ttyUSB0","time":"2023-01-01T12:00:00.000Z","hdop":1.2,"satellites":[{"PRN":1,"ss":42,"used":true},{"PRN":2,"ss":38,"used":true}]}"#;
        
        parse_gpsd_json(&mut data, json).unwrap();
        
        assert_eq!(data.satellites, Some(2));
        assert_eq!(data.hdop, Some(1.2));
    }

    #[test]
    fn test_invalid_json() {
        let mut data = GpsData::new();
        let invalid_json = r#"{"invalid": json"#;
        
        let result = parse_gpsd_json(&mut data, invalid_json);
        assert!(result.is_err());
    }
}
