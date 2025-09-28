// src/gps/nmea.rs
//! NMEA sentence parsing

use super::data::{GpsData, SatelliteInfo};

/// Parse a single NMEA sentence and update GPS data
pub fn parse_nmea_sentence(data: &mut GpsData, line: &str) {
    let parts: Vec<&str> = line.split(',').collect();

    if line.starts_with("$GPGGA") || line.starts_with("$GNGGA") {
        parse_gpgga(data, &parts);
    } else if line.starts_with("$GPRMC") || line.starts_with("$GNRMC") {
        parse_gprmc(data, &parts);
    } else if line.starts_with("$GPGSV") || line.starts_with("$GLGSV") || line.starts_with("$GAGSV") || line.starts_with("$GBGSV") {
        parse_gsv(data, &parts, line);
    }
}

/// Parse GPGGA (Global Positioning System Fix Data) sentence
fn parse_gpgga(data: &mut GpsData, parts: &[&str]) {
    if parts.len() < 15 {
        return;
    }

    // Latitude (field 2 and 3)
    if !parts[2].is_empty() && !parts[3].is_empty() {
        if let Ok(lat) = parts[2].parse::<f64>() {
            let lat_deg = (lat / 100.0) as i32;
            let lat_min = lat % 100.0;
            let mut latitude = lat_deg as f64 + lat_min / 60.0;
            if parts[3] == "S" {
                latitude = -latitude;
            }
            data.latitude = Some(latitude);
        }
    }

    // Longitude (field 4 and 5)
    if !parts[4].is_empty() && !parts[5].is_empty() {
        if let Ok(lon) = parts[4].parse::<f64>() {
            let lon_deg = (lon / 100.0) as i32;
            let lon_min = lon % 100.0;
            let mut longitude = lon_deg as f64 + lon_min / 60.0;
            if parts[5] == "W" {
                longitude = -longitude;
            }
            data.longitude = Some(longitude);
        }
    }

    // Fix quality (field 6)
    if !parts[6].is_empty() {
        if let Ok(quality) = parts[6].parse::<u8>() {
            data.fix_quality = Some(quality);
        }
    }

    // Number of satellites (field 7)
    if !parts[7].is_empty() {
        if let Ok(sats) = parts[7].parse::<u8>() {
            data.satellites = Some(sats);
        }
    }

    // HDOP (field 8)
    if !parts[8].is_empty() {
        if let Ok(hdop) = parts[8].parse::<f64>() {
            data.hdop = Some(hdop);
        }
    }

    // Altitude (field 9)
    if !parts[9].is_empty() {
        if let Ok(alt) = parts[9].parse::<f64>() {
            data.altitude = Some(alt);
        }
    }
}

/// Parse GPRMC (Recommended Minimum Course) sentence
fn parse_gprmc(data: &mut GpsData, parts: &[&str]) {
    if parts.len() < 10 {
        return;
    }

    // Speed over ground in knots (field 7)
    if !parts[7].is_empty() {
        if let Ok(speed_knots) = parts[7].parse::<f64>() {
            data.speed = Some(speed_knots * 1.852); // Convert knots to km/h
        }
    }

    // Course over ground in degrees (field 8)
    if !parts[8].is_empty() {
        if let Ok(course) = parts[8].parse::<f64>() {
            data.course = Some(course);
        }
    }
}

/// Parse GSV (Satellites in View) sentence
fn parse_gsv(data: &mut GpsData, parts: &[&str], line: &str) {
    if parts.len() < 4 {
        return;
    }

    // Determine constellation from sentence type
    let constellation = if line.starts_with("$GPGSV") {
        "GPS"
    } else if line.starts_with("$GLGSV") {
        "GLONASS"
    } else if line.starts_with("$GAGSV") {
        "GALILEO"
    } else if line.starts_with("$GBGSV") {
        "BEIDOU"
    } else {
        "UNKNOWN"
    };

    // Parse message number and total messages
    let message_num = parts[2].parse::<u8>().unwrap_or(0);
    let _total_messages = parts[1].parse::<u8>().unwrap_or(0);

    // If this is the first message, clear existing satellites for this constellation
    if message_num == 1 {
        data.satellites_info.retain(|sat| sat.constellation != constellation);
    }

    // Parse satellite information (up to 4 satellites per message)
    let mut sat_index = 4; // Start after header fields
    while sat_index + 3 < parts.len() {
        if let Ok(prn) = parts[sat_index].parse::<u8>() {
            let mut sat_info = SatelliteInfo::new(prn);
            sat_info.constellation = constellation.to_string();

            // Elevation
            if !parts[sat_index + 1].is_empty() {
                sat_info.elevation = parts[sat_index + 1].parse::<f32>().ok();
            }

            // Azimuth
            if !parts[sat_index + 2].is_empty() {
                sat_info.azimuth = parts[sat_index + 2].parse::<f32>().ok();
            }

            // SNR (may be empty)
            if sat_index + 3 < parts.len() && !parts[sat_index + 3].is_empty() {
                // Remove checksum if present
                let snr_str = parts[sat_index + 3].split('*').next().unwrap_or(parts[sat_index + 3]);
                sat_info.snr = snr_str.parse::<f32>().ok();
            }

            // Add or update satellite info
            if let Some(existing) = data.satellites_info.iter_mut().find(|s| s.prn == prn) {
                *existing = sat_info;
            } else {
                data.satellites_info.push(sat_info);
            }
        }

        sat_index += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpgga_parsing() {
        let mut data = GpsData::new();
        let gpgga = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";
        
        parse_nmea_sentence(&mut data, gpgga);
        
        assert!(data.latitude.is_some());
        assert!(data.longitude.is_some());
        assert_eq!(data.satellites, Some(8));
        assert_eq!(data.hdop, Some(0.9));
        assert_eq!(data.altitude, Some(545.4));
        assert_eq!(data.fix_quality, Some(1));
    }

    #[test]
    fn test_gprmc_parsing() {
        let mut data = GpsData::new();
        let gprmc = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        
        parse_nmea_sentence(&mut data, gprmc);
        
        assert!(data.speed.is_some());
        assert!(data.course.is_some());
        // Speed should be converted from knots to km/h
        assert!((data.speed.unwrap() - 41.5).abs() < 0.1);
        assert_eq!(data.course, Some(84.4));
    }

    #[test]
    fn test_gsv_parsing() {
        let mut data = GpsData::new();
        let gsv = "$GPGSV,3,1,12,01,40,083,46,02,17,308,41,12,07,344,39,14,22,228,45*75";
        
        parse_nmea_sentence(&mut data, gsv);
        
        assert_eq!(data.satellites_info.len(), 4);
        assert_eq!(data.satellites_info[0].prn, 1);
        assert_eq!(data.satellites_info[0].constellation, "GPS");
        assert_eq!(data.satellites_info[0].elevation, Some(40.0));
        assert_eq!(data.satellites_info[0].azimuth, Some(83.0));
        assert_eq!(data.satellites_info[0].snr, Some(46.0));
    }

    #[test]
    fn test_invalid_sentence() {
        let mut data = GpsData::new();
        let invalid = "$INVALID,123,456";
        
        parse_nmea_sentence(&mut data, invalid);
        
        // Should not crash and should not set any values
        assert!(data.latitude.is_none());
        assert!(data.longitude.is_none());
    }
}
