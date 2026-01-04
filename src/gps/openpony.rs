// src/gps/openpony.rs
//! OpenPonyLogger WebSocket integration
//!
//! This module provides WebSocket client functionality to connect to OpenPonyLogger
//! and receive real-time GPS + IMU telemetry data.

use super::data::{GpsData, SatelliteInfo};
use anyhow::Result;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type WebSocketSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type WebSocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Satellite detail from OpenPonyLogger
#[derive(Debug, Deserialize)]
pub struct SatelliteDetail {
    pub prn: u8,
    pub elevation: i32,
    pub azimuth: i32,
    pub snr: Option<i32>,
}

/// OpenPonyLogger telemetry JSON structure
#[derive(Debug, Deserialize)]
pub struct OpenPonyTelemetry {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub alt: Option<f64>,
    pub speed: Option<f64>,    // MPH from OpenPonyLogger
    pub track: Option<f64>,    // GPS course over ground
    pub heading: Option<f64>,  // Magnetometer heading
    pub satellites: Option<u8>,

    // IMU data - accelerometer (g-force)
    pub gx: Option<f64>,
    pub gy: Option<f64>,
    pub gz: Option<f64>,

    // IMU data - gyroscope (deg/s)
    pub rx: Option<f64>,
    pub ry: Option<f64>,
    pub rz: Option<f64>,

    // Satellite details (sent periodically for skyplot)
    pub satellite_details: Option<Vec<SatelliteDetail>>,
}

/// Connect to OpenPonyLogger WebSocket server
pub async fn connect_openpony(url: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}

/// Parse OpenPonyLogger JSON telemetry and update GpsData
pub fn parse_openpony_json(data: &mut GpsData, json: &str) -> Result<()> {
    let telemetry: OpenPonyTelemetry = serde_json::from_str(json)?;

    // GPS position data
    data.latitude = telemetry.lat;
    data.longitude = telemetry.lon;
    data.altitude = telemetry.alt;

    // Speed - convert MPH to km/h
    data.speed = telemetry.speed.map(|s| s * 1.60934);

    // GPS track (COG) and compass heading
    data.course = telemetry.track;
    data.compass_heading = telemetry.heading;

    // Satellite count
    data.satellites = telemetry.satellites;

    // IMU data - accelerometer
    if let (Some(gx), Some(gy), Some(gz)) = (telemetry.gx, telemetry.gy, telemetry.gz) {
        data.acceleration = Some((gx, gy, gz));
    }

    // IMU data - gyroscope
    if let (Some(rx), Some(ry), Some(rz)) = (telemetry.rx, telemetry.ry, telemetry.rz) {
        data.rotation = Some((rx, ry, rz));
    }

    // Satellite details (sent periodically for skyplot visualization)
    if let Some(sat_details) = telemetry.satellite_details {
        data.satellites_info = sat_details
            .into_iter()
            .map(|sat| {
                let mut info = SatelliteInfo::new(sat.prn);
                info.elevation = Some(sat.elevation as f32);
                info.azimuth = Some(sat.azimuth as f32);
                info.snr = sat.snr.map(|s| s as f32);
                // Mark satellites with SNR as "used" in fix
                info.used = sat.snr.is_some();
                info
            })
            .collect();
    }

    // Set source
    data.set_source("OpenPonyLogger");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_telemetry() {
        let json = r#"{
            "lat": 37.7749,
            "lon": -122.4194,
            "alt": 50.0,
            "speed": 25.5,
            "track": 90.0,
            "heading": 92.5,
            "satellites": 12,
            "gx": 0.05,
            "gy": -0.02,
            "gz": 1.0,
            "rx": 2.5,
            "ry": -1.0,
            "rz": 15.0
        }"#;

        let mut data = GpsData::new();
        parse_openpony_json(&mut data, json).unwrap();

        assert_eq!(data.latitude, Some(37.7749));
        assert_eq!(data.longitude, Some(-122.4194));
        assert_eq!(data.altitude, Some(50.0));
        assert!((data.speed.unwrap() - 41.03).abs() < 0.01); // 25.5 MPH to km/h
        assert_eq!(data.course, Some(90.0));
        assert_eq!(data.compass_heading, Some(92.5));
        assert_eq!(data.satellites, Some(12));
        assert_eq!(data.acceleration, Some((0.05, -0.02, 1.0)));
        assert_eq!(data.rotation, Some((2.5, -1.0, 15.0)));
    }

    #[test]
    fn test_parse_partial_telemetry() {
        let json = r#"{
            "lat": 37.7749,
            "lon": -122.4194,
            "speed": 10.0
        }"#;

        let mut data = GpsData::new();
        parse_openpony_json(&mut data, json).unwrap();

        assert_eq!(data.latitude, Some(37.7749));
        assert_eq!(data.longitude, Some(-122.4194));
        assert!(data.speed.is_some());
        assert_eq!(data.acceleration, None);
        assert_eq!(data.rotation, None);
    }
}
