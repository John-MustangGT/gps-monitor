// src/map/tile_cache.rs v2
//! OpenStreetMap tile downloading and caching with resource management

use crate::error::{Result, GpsError};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

/// Calculate tile coordinates from lat/lon and zoom level
pub fn lat_lon_to_tile(lat: f64, lon: f64, zoom: u8) -> (u32, u32) {
    let n = 2_f64.powi(zoom as i32);
    let x = ((lon + 180.0) / 360.0 * n).floor() as u32;
    let lat_rad = lat.to_radians();
    let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n).floor() as u32;
    (x, y)
}

/// Calculate lat/lon from tile coordinates
pub fn tile_to_lat_lon(x: u32, y: u32, zoom: u8) -> (f64, f64) {
    let n = 2_f64.powi(zoom as i32);
    let lon = x as f64 / n * 360.0 - 180.0;
    let lat_rad = ((1.0 - 2.0 * y as f64 / n) * std::f64::consts::PI).sinh().atan();
    let lat = lat_rad.to_degrees();
    (lat, lon)
}

#[derive(Clone)]
pub struct TileCache {
    cache_dir: PathBuf,
    memory_cache: Arc<Mutex<HashMap<(u8, u32, u32), Arc<Vec<u8>>>>>,
    downloading: Arc<Mutex<HashSet<(u8, u32, u32)>>>,
    max_memory_tiles: usize,
    max_concurrent_downloads: usize,
}

impl TileCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| GpsError::Other(format!("Failed to create cache directory: {}", e)))?;

        Ok(Self {
            cache_dir,
            memory_cache: Arc::new(Mutex::new(HashMap::new())),
            downloading: Arc::new(Mutex::new(HashSet::new())),
            max_memory_tiles: 100,
            max_concurrent_downloads: 4,
        })
    }

    /// Get tile from cache or download
    pub fn get_tile(&self, zoom: u8, x: u32, y: u32) -> Result<Arc<Vec<u8>>> {
        let key = (zoom, x, y);

        // Check memory cache first
        {
            let cache = self.memory_cache.lock().unwrap();
            if let Some(tile) = cache.get(&key) {
                return Ok(Arc::clone(tile));
            }
        }

        // Check disk cache
        let path = self.get_tile_path(zoom, x, y);
        if path.exists() {
            let bytes = std::fs::read(&path)
                .map_err(|e| GpsError::Other(format!("Failed to read cached tile: {}", e)))?;
            let tile = Arc::new(bytes);
            self.add_to_memory_cache(key, Arc::clone(&tile));
            return Ok(tile);
        }

        // Not in cache, need to download
        Err(GpsError::Other("Tile not in cache".to_string()))
    }

    /// Download tile in background (non-blocking) with concurrency limit
    pub fn download_tile_async(&self, zoom: u8, x: u32, y: u32) {
        let key = (zoom, x, y);

        // Check if already downloading
        {
            let mut downloading = self.downloading.lock().unwrap();
            
            // Limit concurrent downloads
            if downloading.len() >= self.max_concurrent_downloads {
                return;
            }
            
            if downloading.contains(&key) {
                return;
            }
            
            downloading.insert(key);
        }

        let cache_dir = self.cache_dir.clone();
        let memory_cache = Arc::clone(&self.memory_cache);
        let downloading = Arc::clone(&self.downloading);

        std::thread::spawn(move || {
            if let Ok(bytes) = Self::download_tile(zoom, x, y) {
                // Save to disk
                let path = Self::tile_path(&cache_dir, zoom, x, y);
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&path, &bytes);

                // Add to memory cache
                let tile = Arc::new(bytes);
                let mut cache = memory_cache.lock().unwrap();
                
                // Limit memory cache size
                if cache.len() >= 100 {
                    // Remove oldest entries
                    if let Some(first_key) = cache.keys().next().cloned() {
                        cache.remove(&first_key);
                    }
                }
                
                cache.insert(key, tile);
            }
            
            // Remove from downloading set
            downloading.lock().unwrap().remove(&key);
        });
    }

    /// Download tile from OpenStreetMap
    fn download_tile(zoom: u8, x: u32, y: u32) -> Result<Vec<u8>> {
        let url = format!("https://tile.openstreetmap.org/{}/{}/{}.png", zoom, x, y);
        
        let client = reqwest::blocking::Client::builder()
            .user_agent("GPSMonitor/1.0 (Rust GPS tracking application)")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| GpsError::Other(format!("HTTP client error: {}", e)))?;

        let response = client.get(&url)
            .send()
            .map_err(|e| GpsError::Other(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(GpsError::Other(format!("HTTP error: {}", response.status())));
        }

        let bytes = response.bytes()
            .map_err(|e| GpsError::Other(format!("Failed to read response: {}", e)))?
            .to_vec();

        // Respect OSM tile usage policy - add small delay
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(bytes)
    }

    fn get_tile_path(&self, zoom: u8, x: u32, y: u32) -> PathBuf {
        Self::tile_path(&self.cache_dir, zoom, x, y)
    }

    fn tile_path(cache_dir: &PathBuf, zoom: u8, x: u32, y: u32) -> PathBuf {
        cache_dir.join(format!("{}/{}/{}.png", zoom, x, y))
    }

    fn add_to_memory_cache(&self, key: (u8, u32, u32), tile: Arc<Vec<u8>>) {
        let mut cache = self.memory_cache.lock().unwrap();
        
        // Simple LRU-like behavior: remove oldest if at capacity
        if cache.len() >= self.max_memory_tiles {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }
        
        cache.insert(key, tile);
    }

    /// Preload tiles around a location (limited to prevent resource exhaustion)
    pub fn preload_area(&self, center_lat: f64, center_lon: f64, zoom: u8, radius: u32) {
        let (center_x, center_y) = lat_lon_to_tile(center_lat, center_lon, zoom);
        
        // Limit preload radius to prevent too many downloads
        let limited_radius = radius.min(2);
        
        for dx in 0..=limited_radius {
            for dy in 0..=limited_radius {
                // Download in all 4 quadrants
                self.download_tile_async(zoom, center_x + dx, center_y + dy);
                if dx > 0 {
                    self.download_tile_async(zoom, center_x - dx, center_y + dy);
                }
                if dy > 0 {
                    self.download_tile_async(zoom, center_x + dx, center_y - dy);
                }
                if dx > 0 && dy > 0 {
                    self.download_tile_async(zoom, center_x - dx, center_y - dy);
                }
            }
        }
    }

    /// Clear memory cache
    pub fn clear_memory_cache(&self) {
        self.memory_cache.lock().unwrap().clear();
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let memory_count = self.memory_cache.lock().unwrap().len();
        
        // Count disk cache files recursively
        let mut disk_count = 0;
        let mut disk_size = 0u64;
        
        fn walk_dir(path: &PathBuf, count: &mut usize, size: &mut u64) {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            *count += 1;
                            *size += metadata.len();
                        } else if metadata.is_dir() {
                            walk_dir(&entry.path(), count, size);
                        }
                    }
                }
            }
        }
        
        walk_dir(&self.cache_dir, &mut disk_count, &mut disk_size);

        CacheStats {
            memory_tiles: memory_count,
            disk_tiles: disk_count,
            disk_size_mb: disk_size as f64 / 1_048_576.0,
        }
    }

    /// Clear entire disk cache
    pub fn clear_disk_cache(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.cache_dir)
            .map_err(|e| GpsError::Other(format!("Failed to clear cache: {}", e)))?;
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| GpsError::Other(format!("Failed to recreate cache directory: {}", e)))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub memory_tiles: usize,
    pub disk_tiles: usize,
    pub disk_size_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_coordinates() {
        // Test known coordinates
        let (x, y) = lat_lon_to_tile(42.438878, -71.119277, 12);
        assert!(x > 0 && y > 0);
        
        // Test conversion back
        let (lat, lon) = tile_to_lat_lon(x, y, 12);
        assert!((lat - 42.438878).abs() < 0.1);
        assert!((lon - (-71.119277)).abs() < 0.1);
    }

    #[test]
    fn test_tile_path() {
        let cache_dir = PathBuf::from("/tmp/tiles");
        let path = TileCache::tile_path(&cache_dir, 12, 1234, 5678);
        assert_eq!(path, PathBuf::from("/tmp/tiles/12/1234/5678.png"));
    }
}
