// src/display/gui/map_window.rs v1
//! Map window with live position, tracks, and waypoints

use crate::{gps::GpsData, waypoint::{WaypointExporter, TrackPoint}, map::TileCache};
use eframe::egui;
use std::collections::HashMap;

const TILE_SIZE: f32 = 256.0;

pub struct MapWindow {
    pub open: bool,
    tile_cache: TileCache,
    zoom: u8,
    center_lat: f64,
    center_lon: f64,
    follow_position: bool,
    loaded_tiles: HashMap<(u8, u32, u32), egui::TextureHandle>,
    show_tracks: bool,
    show_waypoints: bool,
    preload_triggered: bool,
}

impl MapWindow {
    pub fn new(tile_cache: TileCache) -> Self {
        Self {
            open: false,
            tile_cache,
            zoom: 13,
            center_lat: 42.438878,
            center_lon: -71.119277,
            follow_position: true,
            loaded_tiles: HashMap::new(),
            show_tracks: true,
            show_waypoints: true,
            preload_triggered: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, gps_data: &GpsData, exporter: &WaypointExporter) {
        if !self.open {
            return;
        }

        // Update center to current position if following
        if self.follow_position {
            if let (Some(lat), Some(lon)) = (gps_data.latitude, gps_data.longitude) {
                self.center_lat = lat;
                self.center_lon = lon;
                
                // Preload tiles around current position (once per opening)
                if !self.preload_triggered {
                    self.tile_cache.preload_area(lat, lon, self.zoom, 2);
                    self.preload_triggered = true;
                }
            }
        }

        egui::Window::new("🗺 Map View")
            .open(&mut self.open)
            .default_size([800.0, 600.0])
            .resizable(true)
            .show(ctx, |ui| {
                // Top controls
                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    if ui.button("➖").clicked() && self.zoom > 1 {
                        self.zoom -= 1;
                        self.preload_triggered = false;
                    }
                    ui.label(format!("{}", self.zoom));
                    if ui.button("➕").clicked() && self.zoom < 18 {
                        self.zoom += 1;
                        self.preload_triggered = false;
                    }

                    ui.separator();

                    ui.checkbox(&mut self.follow_position, "📍 Follow GPS");
                    
                    ui.separator();
                    
                    ui.checkbox(&mut self.show_tracks, "Show Tracks");
                    ui.checkbox(&mut self.show_waypoints, "Show Waypoints");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let stats = self.tile_cache.get_stats();
                        ui.label(format!("Cache: {} tiles ({:.1} MB)", 
                            stats.disk_tiles, stats.disk_size_mb));
                        
                        if ui.button("🗑 Clear Cache").clicked() {
                            let _ = self.tile_cache.clear_disk_cache();
                            self.tile_cache.clear_memory_cache();
                            self.loaded_tiles.clear();
                        }
                    });
                });

                ui.separator();

                // Map display area
                let available_size = ui.available_size();
                let (response, painter) = ui.allocate_painter(available_size, egui::Sense::drag());

                // Handle dragging
                if response.dragged() && !self.follow_position {
                    let delta = response.drag_delta();
                    self.pan_map(delta, available_size.x, available_size.y);
                }

                // Render map
                self.render_map(ctx, &painter, response.rect, gps_data, exporter);

                // Show current coordinates
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Center: {:.6}, {:.6}", self.center_lat, self.center_lon));
                    if let (Some(lat), Some(lon)) = (gps_data.latitude, gps_data.longitude) {
                        ui.separator();
                        ui.label(format!("GPS: {:.6}, {:.6}", lat, lon));
                    }
                });
            });
    }

    fn render_map(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        rect: egui::Rect,
        gps_data: &GpsData,
        exporter: &WaypointExporter,
    ) {
        let width = rect.width();
        let height = rect.height();

        // Calculate which tiles to display
        let (center_tile_x, center_tile_y) = crate::map::lat_lon_to_tile(self.center_lat, self.center_lon, self.zoom);
        
        // Calculate pixel offset within center tile
        let n = 2_f64.powi(self.zoom as i32);
        let center_pixel_x = ((self.center_lon + 180.0) / 360.0 * n * TILE_SIZE as f64) % TILE_SIZE as f64;
        let lat_rad = self.center_lat.to_radians();
        let center_pixel_y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n * TILE_SIZE as f64) % TILE_SIZE as f64;

        // Calculate how many tiles we need in each direction
        let tiles_x = (width / TILE_SIZE).ceil() as i32 + 1;
        let tiles_y = (height / TILE_SIZE).ceil() as i32 + 1;

        // Render tiles
        for dy in -tiles_y..=tiles_y {
            for dx in -tiles_x..=tiles_x {
                let tile_x = (center_tile_x as i32 + dx) as u32;
                let tile_y = (center_tile_y as i32 + dy) as u32;

                // Calculate tile position on screen
                let screen_x = rect.left() + width / 2.0 + dx as f32 * TILE_SIZE - center_pixel_x as f32;
                let screen_y = rect.top() + height / 2.0 + dy as f32 * TILE_SIZE - center_pixel_y as f32;

                self.render_tile(ctx, painter, self.zoom, tile_x, tile_y, screen_x, screen_y);
            }
        }

        // Render GPS position
        if let (Some(lat), Some(lon)) = (gps_data.latitude, gps_data.longitude) {
            if let Some(pos) = self.lat_lon_to_screen(lat, lon, rect) {
                // Draw position circle
                painter.circle_filled(pos, 8.0, egui::Color32::from_rgb(0, 122, 255));
                painter.circle_stroke(pos, 8.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                
                // Draw heading indicator if course available
                if let Some(course) = gps_data.course {
                    let angle = course.to_radians();
                    let end_pos = pos + egui::vec2(angle.sin() as f32 * 15.0, -angle.cos() as f32 * 15.0);
                    painter.line_segment([pos, end_pos], egui::Stroke::new(3.0, egui::Color32::WHITE));
                }
            }
        }

        // Render tracks
        if self.show_tracks {
            for track in exporter.get_tracks() {
                for segment in &track.segments {
                    self.render_track_segment(painter, segment, rect);
                }
            }
        }

        // Render waypoints
        if self.show_waypoints {
            for waypoint in exporter.get_waypoints() {
                if let Some(pos) = self.lat_lon_to_screen(waypoint.latitude, waypoint.longitude, rect) {
                    // Draw waypoint marker
                    painter.circle_filled(pos, 6.0, egui::Color32::RED);
                    painter.circle_stroke(pos, 6.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                    
                    // Draw label
                    let label_pos = pos + egui::vec2(10.0, -10.0);
                    painter.text(
                        label_pos,
                        egui::Align2::LEFT_BOTTOM,
                        &waypoint.name,
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE
                    );
                }
            }
        }
    }

    fn render_tile(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        zoom: u8,
        x: u32,
        y: u32,
        screen_x: f32,
        screen_y: f32,
    ) {
        let key = (zoom, x, y);

        // Check if we already have this tile as a texture
        if let Some(texture) = self.loaded_tiles.get(&key) {
            let rect = egui::Rect::from_min_size(
                egui::pos2(screen_x, screen_y),
                egui::vec2(TILE_SIZE, TILE_SIZE),
            );
            painter.image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
            return;
        }

        // Try to get tile from cache
        match self.tile_cache.get_tile(zoom, x, y) {
            Ok(tile_data) => {
                // Load image
                if let Ok(image) = image::load_from_memory(&tile_data) {
                    let size = [image.width() as usize, image.height() as usize];
                    let rgba = image.to_rgba8();
                    let pixels = rgba.as_flat_samples();
                    
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        pixels.as_slice(),
                    );

                    let texture = ctx.load_texture(
                        format!("tile_{}_{_{}}", zoom, x, y),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );

                    let rect = egui::Rect::from_min_size(
                        egui::pos2(screen_x, screen_y),
                        egui::vec2(TILE_SIZE, TILE_SIZE),
                    );
                    painter.image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

                    self.loaded_tiles.insert(key, texture);
                }
            }
            Err(_) => {
                // Tile not in cache, download it
                self.tile_cache.download_tile_async(zoom, x, y);
                
                // Draw placeholder
                let rect = egui::Rect::from_min_size(
                    egui::pos2(screen_x, screen_y),
                    egui::vec2(TILE_SIZE, TILE_SIZE),
                );
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(240));
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Loading...",
                    egui::FontId::proportional(12.0),
                    egui::Color32::GRAY,
                );
            }
        }
    }

    fn render_track_segment(&self, painter: &egui::Painter, segment: &crate::waypoint::TrackSegment, rect: egui::Rect) {
        let points: Vec<egui::Pos2> = segment.points.iter()
            .filter_map(|pt| self.lat_lon_to_screen(pt.latitude, pt.longitude, rect))
            .collect();

        if points.len() > 1 {
            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)),
            ));
        }
    }

    fn lat_lon_to_screen(&self, lat: f64, lon: f64, rect: egui::Rect) -> Option<egui::Pos2> {
        let n = 2_f64.powi(self.zoom as i32);
        
        // Convert to pixel coordinates
        let world_x = (lon + 180.0) / 360.0 * n * TILE_SIZE as f64;
        let lat_rad = lat.to_radians();
        let world_y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n * TILE_SIZE as f64;

        // Convert center to world coordinates
        let center_world_x = (self.center_lon + 180.0) / 360.0 * n * TILE_SIZE as f64;
        let center_lat_rad = self.center_lat.to_radians();
        let center_world_y = (1.0 - (center_lat_rad.tan() + 1.0 / center_lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n * TILE_SIZE as f64;

        // Calculate screen position
        let screen_x = rect.left() + rect.width() / 2.0 + (world_x - center_world_x) as f32;
        let screen_y = rect.top() + rect.height() / 2.0 + (world_y - center_world_y) as f32;

        // Check if on screen
        if screen_x >= rect.left() && screen_x <= rect.right() &&
           screen_y >= rect.top() && screen_y <= rect.bottom() {
            Some(egui::pos2(screen_x, screen_y))
        } else {
            None
        }
    }

    fn pan_map(&mut self, delta: egui::Vec2, width: f32, height: f32) {
        let n = 2_f64.powi(self.zoom as i32);
        let pixels_per_degree_lon = n * TILE_SIZE as f64 / 360.0;
        
        let lat_rad = self.center_lat.to_radians();
        let pixels_per_degree_lat = n * TILE_SIZE as f64 * lat_rad.cos() / 360.0;

        self.center_lon -= (delta.x / pixels_per_degree_lon as f32) as f64;
        self.center_lat -= (delta.y / pixels_per_degree_lat as f32) as f64;

        // Clamp coordinates
        self.center_lat = self.center_lat.clamp(-85.0, 85.0);
        self.center_lon = ((self.center_lon + 180.0) % 360.0) - 180.0;
    }

    pub fn on_close(&mut self) {
        self.preload_triggered = false;
    }
}
