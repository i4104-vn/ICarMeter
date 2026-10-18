// NMEA 0183 protocol parser for GPS speed readings ($GPRMC, $GPVTG)

/// Parses NMEA sentence ($GPRMC, $GNRMC, $GPVTG, $GNVTG) for vehicle speed in km/h
pub fn parse_nmea_speed(line: &str) -> Option<f32> {
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
        // Field 2: Status ('A' = Active/Valid, 'V' = Void/Invalid)
        if parts[2] == "A" {
            // Field 7: Speed over ground in Knots
            if let Ok(knots) = parts[7].parse::<f32>() {
                // 1 knot = 1.852 km/h
                return Some(knots * 1.852);
            }
        }
    }

    // VTG sentences: $GPVTG, $GNVTG
    if sentence_type.ends_with("VTG") && parts.len() > 7 {
        // Field 7: Speed in km/h followed by 'K' in field 8
        if parts.len() > 8
            && parts[8].starts_with('K')
            && let Ok(kmh) = parts[7].parse::<f32>()
        {
            return Some(kmh);
        }
    }

    None
}
