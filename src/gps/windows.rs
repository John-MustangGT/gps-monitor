// src/gps/windows.rs
//! Windows Location Services integration

#[cfg(windows)]
use {
    super::data::GpsData,
    crate::error::{Result, GpsError},
    std::time::Duration,
    tokio::time::sleep,
    windows::{
        Devices::Geolocation::*,
        Foundation::*,
    },
};

#[cfg(windows)]
/// Initialize Windows Location Services
pub async fn request_location_access() -> Result<()> {
    let access_status = Geolocator::RequestAccessAsync()?.await?;
    
    match access_status {
        GeolocationAccessStatus::Allowed => {
            println!("Location access granted!");
            Ok(())
        }
        GeolocationAccessStatus::Denied => {
            Err(GpsError::Other("Location access denied by user".to_string()))
        }
        GeolocationAccessStatus::Unspecified => {
            Err(GpsError::Other("Location access unspecified".to_string()))
        }
        _ => {
            Err(GpsError::Other("Unknown location access status".to_string()))
        }
    }
}

#[cfg(windows)]
/// Create and configure a Windows Geolocator
pub fn create_geolocator(accuracy: u32) -> Result<Geolocator> {
    let geolocator = Geolocator::new()?;
    
    // Set desired accuracy
    let desired_accuracy = match accuracy {
        0..=100 => PositionAccuracy::High,
        _ => PositionAccuracy::Default,
    };
    geolocator.SetDesiredAccuracy(desired_accuracy)?;

    // Set movement threshold (1 meter)
    geolocator.SetMovementThreshold(1.0)?;
    
    Ok(geolocator)
}

#[cfg(windows)]
/// Get current position from Windows Location Services
pub async fn get_position(geolocator: &Geolocator) -> Result<Geoposition> {
    // Set timeout for position request (10 seconds)
    let timeout = TimeSpan {
        Duration: 10_000_000 * 10, // 10 seconds in 100ns units
    };
    
    let position = geolocator
        .GetGeopositionAsyncWithAgeAndTimeout(timeout, timeout)?
        .await?;
    
    Ok(position)
}

#[cfg(windows)]
/// Update GPS data from Windows Geoposition
pub fn update_from_position(data: &mut GpsData, position: &Geoposition) -> Result<()> {
    data.update_timestamp();
    data.set_source("Windows Location");
    
    // Extract coordinate data
    if let Ok(coordinate) = position.Coordinate() {
        if let Ok(point) = coordinate.Point() {
            if let Ok(pos) = point.Position() {
                data.latitude = Some(pos.Latitude);
                data.longitude = Some(pos.Longitude);
                
                // Altitude (optional) - it's a direct f64 value, not a Result
                let alt = pos.Altitude;
                if alt != 0.0 {
                    data.altitude = Some(alt);
                }
            }
        }
        
        // Accuracy
        if let Ok(acc) = coordinate.Accuracy() {
            data.accuracy = Some(acc);
        }
        
        // Heading (optional)
        if let Ok(heading) = coordinate.Heading() {
            if let Ok(h) = heading.Value() {
                data.course = Some(h);
            }
        }
        
        // Speed (optional)
        if let Ok(speed) = coordinate.Speed() {
            if let Ok(s) = speed.Value() {
                data.speed = Some(s * 3.6); // Convert m/s to km/h
            }
        }
    }
    
    // Get source information for raw data display
    // Note: Geoposition doesn't have a Source() method in newer Windows API
    // We'll just use a generic source string
    data.raw_data = format!(
        "Source: Windows Location Service, Accuracy: {:.1}m", 
        data.accuracy.unwrap_or(0.0)
    );
    
    Ok(())
}

#[cfg(windows)]
/// Run Windows Location Services monitoring loop
pub async fn run_location_monitoring(
    geolocator: Geolocator,
    data: std::sync::Arc<std::sync::RwLock<GpsData>>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    interval: u64,
) {
    use std::sync::atomic::Ordering;
    
    while running.load(Ordering::Relaxed) {
        match get_position(&geolocator).await {
            Ok(position) => {
                let mut data_guard = data.write().unwrap();
                if let Err(e) = update_from_position(&mut data_guard, &position) {
                    eprintln!("Error updating position data: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error getting Windows location: {}", e);
            }
        }
        
        sleep(Duration::from_secs(interval)).await;
    }
}

// Non-Windows implementations
#[cfg(not(windows))]
pub async fn request_location_access() -> Result<()> {
    Err(GpsError::Other("Windows Location Service is only available on Windows".to_string()))
}

#[cfg(not(windows))]
pub fn create_geolocator(_accuracy: u32) -> Result<()> {
    Err(GpsError::Other("Windows Location Service is only available on Windows".to_string()))
}