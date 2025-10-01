// src/waypoint.rs v1
//! Waypoint recording and export functionality

use crate::gps::GpsData;
use crate::error::{Result, GpsError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
}

impl Waypoint {
    pub fn from_gps_data(gps_data: &GpsData, name: String, description: Option<String>) -> Option<Self> {
        if let (Some(lat), Some(lon)) = (gps_data.latitude, gps_data.longitude) {
            Some(Self {
                name,
                latitude: lat,
                longitude: lon,
                elevation: gps_data.altitude,
                timestamp: gps_data.timestamp.unwrap_or_else(Utc::now),
                description,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaypointFormat {
    GPX,
    GeoJSON,
    KML,
    CSV,
}

impl WaypointFormat {
    pub fn extension(&self) -> &str {
        match self {
            WaypointFormat::GPX => "gpx",
            WaypointFormat::GeoJSON => "geojson",
            WaypointFormat::KML => "kml",
            WaypointFormat::CSV => "csv",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            WaypointFormat::GPX => "GPX (GPS Exchange)",
            WaypointFormat::GeoJSON => "GeoJSON",
            WaypointFormat::KML => "KML (Keyhole)",
            WaypointFormat::CSV => "CSV",
        }
    }
}

pub struct WaypointExporter {
    waypoints: Vec<Waypoint>,
}

impl WaypointExporter {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    pub fn waypoint_count(&self) -> usize {
        self.waypoints.len()
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn export_to_file(&self, path: &Path, format: WaypointFormat) -> Result<()> {
        if self.waypoints.is_empty() {
            return Err(GpsError::Other("No waypoints to export".to_string()));
        }

        let content = match format {
            WaypointFormat::GPX => self.to_gpx(),
            WaypointFormat::GeoJSON => self.to_geojson()?,
            WaypointFormat::KML => self.to_kml(),
            WaypointFormat::CSV => self.to_csv(),
        };

        let mut file = File::create(path)
            .map_err(|e| GpsError::Io(e))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| GpsError::Io(e))?;

        Ok(())
    }

    fn to_gpx(&self) -> String {
        let mut gpx = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="GPS Monitor" xmlns="http://www.topografix.com/GPX/1/1">
"#);

        for waypoint in &self.waypoints {
            gpx.push_str(&format!(
                r#"  <wpt lat="{}" lon="{}">
    <name>{}</name>
"#,
                waypoint.latitude,
                waypoint.longitude,
                Self::escape_xml(&waypoint.name)
            ));

            if let Some(ele) = waypoint.elevation {
                gpx.push_str(&format!("    <ele>{}</ele>\n", ele));
            }

            gpx.push_str(&format!(
                "    <time>{}</time>\n",
                waypoint.timestamp.to_rfc3339()
            ));

            if let Some(ref desc) = waypoint.description {
                gpx.push_str(&format!(
                    "    <desc>{}</desc>\n",
                    Self::escape_xml(desc)
                ));
            }

            gpx.push_str("  </wpt>\n");
        }

        gpx.push_str("</gpx>\n");
        gpx
    }

    fn to_geojson(&self) -> Result<String> {
        let features: Vec<serde_json::Value> = self.waypoints.iter().map(|wp| {
            let mut properties = serde_json::json!({
                "name": wp.name,
                "timestamp": wp.timestamp.to_rfc3339(),
            });

            if let Some(ele) = wp.elevation {
                properties["elevation"] = serde_json::json!(ele);
            }

            if let Some(ref desc) = wp.description {
                properties["description"] = serde_json::json!(desc);
            }

            serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [wp.longitude, wp.latitude, wp.elevation.unwrap_or(0.0)]
                },
                "properties": properties
            })
        }).collect();

        let feature_collection = serde_json::json!({
            "type": "FeatureCollection",
            "features": features
        });

        serde_json::to_string_pretty(&feature_collection)
            .map_err(|e| GpsError::Json(e))
    }

    fn to_kml(&self) -> String {
        let mut kml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <name>GPS Monitor Waypoints</name>
"#);

        for waypoint in &self.waypoints {
            kml.push_str("    <Placemark>\n");
            kml.push_str(&format!(
                "      <name>{}</name>\n",
                Self::escape_xml(&waypoint.name)
            ));

            if let Some(ref desc) = waypoint.description {
                kml.push_str(&format!(
                    "      <description>{}</description>\n",
                    Self::escape_xml(desc)
                ));
            }

            kml.push_str(&format!(
                "      <TimeStamp><when>{}</when></TimeStamp>\n",
                waypoint.timestamp.to_rfc3339()
            ));

            kml.push_str("      <Point>\n");
            kml.push_str(&format!(
                "        <coordinates>{},{},{}</coordinates>\n",
                waypoint.longitude,
                waypoint.latitude,
                waypoint.elevation.unwrap_or(0.0)
            ));
            kml.push_str("      </Point>\n");
            kml.push_str("    </Placemark>\n");
        }

        kml.push_str("  </Document>\n</kml>\n");
        kml
    }

    fn to_csv(&self) -> String {
        let mut csv = String::from("name,latitude,longitude,elevation,timestamp,description\n");

        for waypoint in &self.waypoints {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                Self::escape_csv(&waypoint.name),
                waypoint.latitude,
                waypoint.longitude,
                waypoint.elevation.map_or(String::new(), |e| e.to_string()),
                waypoint.timestamp.to_rfc3339(),
                waypoint.description.as_ref().map_or(String::new(), |d| Self::escape_csv(d))
            ));
        }

        csv
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn escape_csv(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }

    pub fn get_waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }
}

impl Default for WaypointExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_creation() {
        let mut gps_data = GpsData::new();
        gps_data.latitude = Some(42.438878);
        gps_data.longitude = Some(-71.119277);
        gps_data.altitude = Some(100.0);

        let waypoint = Waypoint::from_gps_data(&gps_data, "Test".to_string(), None);
        assert!(waypoint.is_some());
        
        let wp = waypoint.unwrap();
        assert_eq!(wp.name, "Test");
        assert_eq!(wp.latitude, 42.438878);
        assert_eq!(wp.longitude, -71.119277);
    }

    #[test]
    fn test_gpx_export() {
        let mut exporter = WaypointExporter::new();
        exporter.add_waypoint(Waypoint {
            name: "Test Point".to_string(),
            latitude: 42.0,
            longitude: -71.0,
            elevation: Some(100.0),
            timestamp: Utc::now(),
            description: Some("Test description".to_string()),
        });

        let gpx = exporter.to_gpx();
        assert!(gpx.contains("<gpx"));
        assert!(gpx.contains("Test Point"));
        assert!(gpx.contains("lat=\"42\""));
    }

    #[test]
    fn test_csv_export() {
        let mut exporter = WaypointExporter::new();
        exporter.add_waypoint(Waypoint {
            name: "Test".to_string(),
            latitude: 42.0,
            longitude: -71.0,
            elevation: None,
            timestamp: Utc::now(),
            description: None,
        });

        let csv = exporter.to_csv();
        assert!(csv.contains("name,latitude,longitude"));
        assert!(csv.contains("Test"));
    }
}
