// src/waypoint.rs v2
//! Waypoint and track recording functionality

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub speed: Option<f64>,      // km/h
    pub course: Option<f64>,     // degrees
    pub hdop: Option<f64>,       // Horizontal dilution of precision
    pub satellites: Option<u8>,  // Number of satellites
    // OBD-II data (optional, for future use)
    pub obd_speed: Option<f64>,     // km/h from OBD-II
    pub obd_rpm: Option<u16>,       // Engine RPM
    pub obd_throttle: Option<f32>,  // Throttle position %
    pub obd_load: Option<f32>,      // Engine load %
    pub obd_temp: Option<i16>,      // Coolant temp °C
}

impl TrackPoint {
    pub fn from_gps_data(gps_data: &GpsData) -> Option<Self> {
        if let (Some(lat), Some(lon)) = (gps_data.latitude, gps_data.longitude) {
            Some(Self {
                latitude: lat,
                longitude: lon,
                elevation: gps_data.altitude,
                timestamp: gps_data.timestamp.unwrap_or_else(Utc::now),
                speed: gps_data.speed,
                course: gps_data.course,
                hdop: gps_data.hdop,
                satellites: gps_data.satellites,
                obd_speed: None,
                obd_rpm: None,
                obd_throttle: None,
                obd_load: None,
                obd_temp: None,
            })
        } else {
            None
        }
    }

    /// Calculate distance to another track point in meters using Haversine formula
    pub fn distance_to(&self, other: &TrackPoint) -> f64 {
        let r = 6371000.0; // Earth radius in meters
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSegment {
    pub points: Vec<TrackPoint>,
}

impl TrackSegment {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
        }
    }

    pub fn add_point(&mut self, point: TrackPoint) {
        self.points.push(point);
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Calculate total distance of segment in meters
    pub fn total_distance(&self) -> f64 {
        self.points.windows(2).map(|w| w[0].distance_to(&w[1])).sum()
    }

    /// Calculate duration of segment
    pub fn duration(&self) -> Option<chrono::Duration> {
        if self.points.len() < 2 {
            return None;
        }
        let start = self.points.first()?.timestamp;
        let end = self.points.last()?.timestamp;
        Some(end.signed_duration_since(start))
    }
}

impl Default for TrackSegment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub segments: Vec<TrackSegment>,
}

impl Track {
    pub fn new(name: String) -> Self {
        Self {
            name,
            segments: vec![TrackSegment::new()],
        }
    }

    pub fn add_point(&mut self, point: TrackPoint) {
        if let Some(segment) = self.segments.last_mut() {
            segment.add_point(point);
        }
    }

    pub fn start_new_segment(&mut self) {
        self.segments.push(TrackSegment::new());
    }

    pub fn total_points(&self) -> usize {
        self.segments.iter().map(|s| s.len()).sum()
    }

    pub fn total_distance(&self) -> f64 {
        self.segments.iter().map(|s| s.total_distance()).sum()
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        if self.segments.is_empty() {
            return None;
        }
        let first_point = self.segments.first()?.points.first()?;
        let last_point = self.segments.last()?.points.last()?;
        Some(last_point.timestamp.signed_duration_since(first_point.timestamp))
    }

    pub fn average_speed(&self) -> Option<f64> {
        let distance = self.total_distance() / 1000.0; // km
        let duration = self.duration()?;
        let hours = duration.num_seconds() as f64 / 3600.0;
        if hours > 0.0 {
            Some(distance / hours)
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
    tracks: Vec<Track>,
}

impl WaypointExporter {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            tracks: Vec::new(),
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn waypoint_count(&self) -> usize {
        self.waypoints.len()
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
        self.tracks.clear();
    }

    pub fn clear_waypoints(&mut self) {
        self.waypoints.clear();
    }

    pub fn clear_tracks(&mut self) {
        self.tracks.clear();
    }

    pub fn export_to_file(&self, path: &Path, format: WaypointFormat) -> Result<()> {
        if self.waypoints.is_empty() && self.tracks.is_empty() {
            return Err(GpsError::Other("No waypoints or tracks to export".to_string()));
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
<gpx version="1.1" creator="GPS Monitor" 
     xmlns="http://www.topografix.com/GPX/1/1"
     xmlns:obd="http://gpsmonitor.com/obd/1.0">
"#);

        // Add waypoints
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

        // Add tracks
        for track in &self.tracks {
            gpx.push_str("  <trk>\n");
            gpx.push_str(&format!("    <name>{}</name>\n", Self::escape_xml(&track.name)));

            for segment in &track.segments {
                if segment.is_empty() {
                    continue;
                }
                
                gpx.push_str("    <trkseg>\n");
                
                for point in &segment.points {
                    gpx.push_str(&format!(
                        "      <trkpt lat=\"{}\" lon=\"{}\">\n",
                        point.latitude, point.longitude
                    ));

                    if let Some(ele) = point.elevation {
                        gpx.push_str(&format!("        <ele>{}</ele>\n", ele));
                    }

                    gpx.push_str(&format!(
                        "        <time>{}</time>\n",
                        point.timestamp.to_rfc3339()
                    ));

                    // Add GPS quality data
                    if point.speed.is_some() || point.course.is_some() || 
                       point.hdop.is_some() || point.satellites.is_some() ||
                       point.obd_speed.is_some() || point.obd_rpm.is_some() {
                        gpx.push_str("        <extensions>\n");

                        if let Some(speed) = point.speed {
                            gpx.push_str(&format!("          <speed>{}</speed>\n", speed / 3.6)); // m/s
                        }

                        if let Some(course) = point.course {
                            gpx.push_str(&format!("          <course>{}</course>\n", course));
                        }

                        if let Some(hdop) = point.hdop {
                            gpx.push_str(&format!("          <hdop>{}</hdop>\n", hdop));
                        }

                        if let Some(sat) = point.satellites {
                            gpx.push_str(&format!("          <sat>{}</sat>\n", sat));
                        }

                        // OBD-II data
                        if point.obd_speed.is_some() || point.obd_rpm.is_some() ||
                           point.obd_throttle.is_some() || point.obd_load.is_some() ||
                           point.obd_temp.is_some() {
                            gpx.push_str("          <obd:vehicle_data>\n");

                            if let Some(speed) = point.obd_speed {
                                gpx.push_str(&format!("            <obd:speed>{}</obd:speed>\n", speed));
                            }

                            if let Some(rpm) = point.obd_rpm {
                                gpx.push_str(&format!("            <obd:rpm>{}</obd:rpm>\n", rpm));
                            }

                            if let Some(throttle) = point.obd_throttle {
                                gpx.push_str(&format!("            <obd:throttle_position>{}</obd:throttle_position>\n", throttle));
                            }

                            if let Some(load) = point.obd_load {
                                gpx.push_str(&format!("            <obd:engine_load>{}</obd:engine_load>\n", load));
                            }

                            if let Some(temp) = point.obd_temp {
                                gpx.push_str(&format!("            <obd:coolant_temp>{}</obd:coolant_temp>\n", temp));
                            }

                            gpx.push_str("          </obd:vehicle_data>\n");
                        }

                        gpx.push_str("        </extensions>\n");
                    }

                    gpx.push_str("      </trkpt>\n");
                }

                gpx.push_str("    </trkseg>\n");
            }

            gpx.push_str("  </trk>\n");
        }

        gpx.push_str("</gpx>\n");
        gpx
    }

    fn to_geojson(&self) -> Result<String> {
        let mut features = Vec::new();

        // Add waypoints as Point features
        for wp in &self.waypoints {
            let mut properties = serde_json::json!({
                "name": wp.name,
                "timestamp": wp.timestamp.to_rfc3339(),
                "type": "waypoint"
            });

            if let Some(ele) = wp.elevation {
                properties["elevation"] = serde_json::json!(ele);
            }

            if let Some(ref desc) = wp.description {
                properties["description"] = serde_json::json!(desc);
            }

            features.push(serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [wp.longitude, wp.latitude, wp.elevation.unwrap_or(0.0)]
                },
                "properties": properties
            }));
        }

        // Add tracks as LineString features
        for track in &self.tracks {
            for segment in &track.segments {
                if segment.is_empty() {
                    continue;
                }

                let coordinates: Vec<serde_json::Value> = segment.points.iter().map(|pt| {
                    serde_json::json!([pt.longitude, pt.latitude, pt.elevation.unwrap_or(0.0)])
                }).collect();

                features.push(serde_json::json!({
                    "type": "Feature",
                    "geometry": {
                        "type": "LineString",
                        "coordinates": coordinates
                    },
                    "properties": {
                        "name": track.name,
                        "type": "track",
                        "points": segment.len()
                    }
                }));
            }
        }

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
    <name>GPS Monitor Data</name>
"#);

        // Add waypoints as Placemarks
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

        // Add tracks as LineStrings
        for track in &self.tracks {
            kml.push_str("    <Placemark>\n");
            kml.push_str(&format!("      <name>{}</name>\n", Self::escape_xml(&track.name)));
            kml.push_str("      <Style>\n");
            kml.push_str("        <LineStyle>\n");
            kml.push_str("          <color>ff0000ff</color>\n");
            kml.push_str("          <width>4</width>\n");
            kml.push_str("        </LineStyle>\n");
            kml.push_str("      </Style>\n");

            for segment in &track.segments {
                if segment.is_empty() {
                    continue;
                }

                kml.push_str("      <LineString>\n");
                kml.push_str("        <coordinates>\n");

                for point in &segment.points {
                    kml.push_str(&format!(
                        "          {},{},{}\n",
                        point.longitude,
                        point.latitude,
                        point.elevation.unwrap_or(0.0)
                    ));
                }

                kml.push_str("        </coordinates>\n");
                kml.push_str("      </LineString>\n");
            }

            kml.push_str("    </Placemark>\n");
        }

        kml.push_str("  </Document>\n</kml>\n");
        kml
    }

    fn to_csv(&self) -> String {
        let mut csv = String::from("type,name,latitude,longitude,elevation,timestamp,description,speed,course,hdop,satellites\n");

        // Add waypoints
        for waypoint in &self.waypoints {
            csv.push_str(&format!(
                "waypoint,{},{},{},{},{},{},,,,,\n",
                Self::escape_csv(&waypoint.name),
                waypoint.latitude,
                waypoint.longitude,
                waypoint.elevation.map_or(String::new(), |e| e.to_string()),
                waypoint.timestamp.to_rfc3339(),
                waypoint.description.as_ref().map_or(String::new(), |d| Self::escape_csv(d))
            ));
        }

        // Add track points
        for track in &self.tracks {
            for segment in &track.segments {
                for point in &segment.points {
                    csv.push_str(&format!(
                        "track,{},{},{},{},{},,,{},{},{},{}\n",
                        Self::escape_csv(&track.name),
                        point.latitude,
                        point.longitude,
                        point.elevation.map_or(String::new(), |e| e.to_string()),
                        point.timestamp.to_rfc3339(),
                        point.speed.map_or(String::new(), |s| s.to_string()),
                        point.course.map_or(String::new(), |c| c.to_string()),
                        point.hdop.map_or(String::new(), |h| h.to_string()),
                        point.satellites.map_or(String::new(), |s| s.to_string())
                    ));
                }
            }
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

    pub fn get_tracks(&self) -> &[Track] {
        &self.tracks
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
    fn test_track_point_distance() {
        let p1 = TrackPoint {
            latitude: 42.0,
            longitude: -71.0,
            elevation: None,
            timestamp: Utc::now(),
            speed: None,
            course: None,
            hdop: None,
            satellites: None,
            obd_speed: None,
            obd_rpm: None,
            obd_throttle: None,
            obd_load: None,
            obd_temp: None,
        };

        let p2 = TrackPoint {
            latitude: 42.01,
            longitude: -71.0,
            elevation: None,
            timestamp: Utc::now(),
            speed: None,
            course: None,
            hdop: None,
            satellites: None,
            obd_speed: None,
            obd_rpm: None,
            obd_throttle: None,
            obd_load: None,
            obd_temp: None,
        };

        let distance = p1.distance_to(&p2);
        assert!(distance > 1100.0 && distance < 1120.0); // ~1.11 km
    }

    #[test]
    fn test_track_statistics() {
        let mut track = Track::new("Test Track".to_string());
        
        let p1 = TrackPoint {
            latitude: 42.0,
            longitude: -71.0,
            elevation: Some(100.0),
            timestamp: Utc::now(),
            speed: Some(50.0),
            course: None,
            hdop: None,
            satellites: None,
            obd_speed: None,
            obd_rpm: None,
            obd_throttle: None,
            obd_load: None,
            obd_temp: None,
        };

        let p2 = TrackPoint {
            latitude: 42.01,
            longitude: -71.0,
            elevation: Some(105.0),
            timestamp: Utc::now() + chrono::Duration::seconds(60),
            speed: Some(55.0),
            course: None,
            hdop: None,
            satellites: None,
            obd_speed: None,
            obd_rpm: None,
            obd_throttle: None,
            obd_load: None,
            obd_temp: None,
        };

        track.add_point(p1);
        track.add_point(p2);

        assert_eq!(track.total_points(), 2);
        assert!(track.total_distance() > 1100.0);
        assert!(track.duration().is_some());
    }
}


