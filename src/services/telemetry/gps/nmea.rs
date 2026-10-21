// NMEA 0183 protocol parser for GPS speed, coordinates, and heading ($GPRMC, $GPVTG)

#[derive(Debug, Clone, Copy, Default)]
pub struct GpsUpdate {
    pub speed_kmh: Option<f32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub heading: Option<f32>,
    pub valid_fix: bool,
}

/// Parses an NMEA sentence ($GPRMC, $GNRMC, $GPVTG, $GNVTG) for full telemetry update
pub fn parse_nmea_update(line: &str) -> Option<GpsUpdate> {
    if !line.starts_with('$') {
        return None;
    }

    let parts: Vec<&str> = line.split(',').collect();
    if parts.is_empty() {
        return None;
    }

    let sentence_type = parts[0];

    // RMC sentences: $GPRMC, $GNRMC, $GARMC, $BDRMC
    if sentence_type.ends_with("RMC") && parts.len() > 7 {
        let is_valid = parts[2] == "A";
        let mut update = GpsUpdate {
            valid_fix: is_valid,
            ..Default::default()
        };

        if is_valid {
            // Field 3 & 4: Latitude
            if parts.len() > 4 && let Some(mut lat) = parse_nmea_degrees(parts[3], false) {
                if parts[4] == "S" {
                    lat = -lat;
                }
                update.latitude = Some(lat);
            }

            // Field 5 & 6: Longitude
            if parts.len() > 6 && let Some(mut lon) = parse_nmea_degrees(parts[5], true) {
                if parts[6] == "W" {
                    lon = -lon;
                }
                update.longitude = Some(lon);
            }

            // Field 7: Speed over ground in Knots (1 knot = 1.852 km/h)
            if let Ok(knots) = parts[7].parse::<f32>() {
                update.speed_kmh = Some(knots * 1.852);
            }

            // Field 8: Track angle in degrees (True Heading)
            if parts.len() > 8 && let Ok(course) = parts[8].parse::<f32>() {
                update.heading = Some(course);
            }

            return Some(update);
        }
    }

    // VTG sentences: $GPVTG, $GNVTG
    if sentence_type.ends_with("VTG") && parts.len() > 7 {
        if parts.len() > 8 && parts[8].starts_with('K') && let Ok(kmh) = parts[7].parse::<f32>() {
            let mut update = GpsUpdate {
                speed_kmh: Some(kmh),
                valid_fix: true,
                ..Default::default()
            };
            if let Ok(course) = parts[1].parse::<f32>() {
                update.heading = Some(course);
            }
            return Some(update);
        }
    }

    None
}

/// Parses NMEA sentence ($GPRMC, $GNRMC, $GPVTG, $GNVTG) for vehicle speed in km/h
#[allow(dead_code)]
pub fn parse_nmea_speed(line: &str) -> Option<f32> {
    parse_nmea_update(line).and_then(|u| u.speed_kmh)
}

fn parse_nmea_degrees(raw: &str, is_lon: bool) -> Option<f64> {
    if raw.is_empty() {
        return None;
    }
    let deg_len = if is_lon { 3 } else { 2 };
    if raw.len() <= deg_len {
        return None;
    }
    let (deg_str, min_str) = raw.split_at(deg_len);
    let deg: f64 = deg_str.parse().ok()?;
    let min: f64 = min_str.parse().ok()?;
    Some(deg + min / 60.0)
}
