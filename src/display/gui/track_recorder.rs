// src/display/gui/track_recorder.rs v1
//! Track recording UI and control

use crate::{gps::GpsData, waypoint::{Track, TrackPoint}};
use chrono::Utc;
use std::time::{Duration, Instant};

pub struct TrackRecorder {
    pub recording: bool,
    current_track: Option<Track>,
    track_name: String,
    last_point_time: Option<Instant>,
    min_distance: f64,      // Minimum distance in meters between points
    min_time: Duration,     // Minimum time between points
    total_points: usize,
    start_time: Option<chrono::DateTime<Utc>>,
}

impl TrackRecorder {
    pub fn new() -> Self {
        Self {
            recording: false,
            current_track: None,
            track_name: String::new(),
            last_point_time: None,
            min_distance: 5.0,      // 5 meters default
            min_time: Duration::from_secs(1), // 1 second default
            total_points: 0,
            start_time: None,
        }
    }

    pub fn start_recording(&mut self, name: String) {
        self.track_name = if name.is_empty() {
            format!("Track {}", Utc::now().format("%Y-%m-%d %H:%M"))
        } else {
            name
        };

        self.current_track = Some(Track::new(self.track_name.clone()));
        self.recording = true;
        self.last_point_time = Some(Instant::now());
        self.total_points = 0;
        self.start_time = Some(Utc::now());
    }

    pub fn stop_recording(&mut self) -> Option<Track> {
        self.recording = false;
        self.last_point_time = None;
        self.current_track.take()
    }

    pub fn pause_recording(&mut self) {
        if self.recording && self.current_track.is_some() {
            // Start a new segment when resumed
            if let Some(ref mut track) = self.current_track {
                track.start_new_segment();
            }
        }
        self.recording = false;
    }

    pub fn update(&mut self, gps_data: &GpsData) {
        if !self.recording || self.current_track.is_none() {
            return;
        }

        // Check if GPS has a fix
        if !gps_data.has_fix() {
            return;
        }

        // Check time threshold
        if let Some(last_time) = self.last_point_time {
            if last_time.elapsed() < self.min_time {
                return;
            }
        }

        // Create track point from GPS data
        if let Some(point) = TrackPoint::from_gps_data(gps_data) {
            // Check distance threshold (if we have a previous point)
            if let Some(ref track) = self.current_track {
                if let Some(segment) = track.segments.last() {
                    if let Some(last_point) = segment.points.last() {
                        let distance = last_point.distance_to(&point);
                        if distance < self.min_distance {
                            return; // Too close to last point
                        }
                    }
                }
            }

            // Add point to current track
            if let Some(ref mut track) = self.current_track {
                track.add_point(point);
                self.total_points += 1;
                self.last_point_time = Some(Instant::now());
            }
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn get_track_name(&self) -> &str {
        &self.track_name
    }

    pub fn get_track_stats(&self) -> Option<TrackStats> {
        let track = self.current_track.as_ref()?;
        let start = self.start_time?;
        let elapsed = Utc::now().signed_duration_since(start);

        Some(TrackStats {
            points: self.total_points,
            distance_km: track.total_distance() / 1000.0,
            duration: elapsed,
            avg_speed: track.average_speed(),
        })
    }

    pub fn set_min_distance(&mut self, meters: f64) {
        self.min_distance = meters.max(0.5); // At least 0.5m
    }

    pub fn set_min_time(&mut self, seconds: u64) {
        self.min_time = Duration::from_secs(seconds.max(1)); // At least 1 second
    }

    pub fn get_min_distance(&self) -> f64 {
        self.min_distance
    }

    pub fn get_min_time_seconds(&self) -> u64 {
        self.min_time.as_secs()
    }
}

impl Default for TrackRecorder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TrackStats {
    pub points: usize,
    pub distance_km: f64,
    pub duration: chrono::Duration,
    pub avg_speed: Option<f64>,
}

impl TrackStats {
    pub fn format_duration(&self) -> String {
        let total_seconds = self.duration.num_seconds();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}
