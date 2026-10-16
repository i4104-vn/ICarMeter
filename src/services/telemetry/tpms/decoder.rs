// Decoding algorithms for wireless BLE TPMS sensor data packets

/// Matches target sensor configuration against incoming MAC address or payload serial bytes
pub fn matches_sensor_id(target: &str, mac: &str, payload: &[u8]) -> bool {
    let target = target.trim();
    if target.is_empty() {
        return false;
    }

    let target_clean = target.replace([':', '-', ' '], "").to_uppercase();
    let mac_clean = mac.replace([':', '-', ' '], "").to_uppercase();

    // 1. Direct or partial MAC match
    if mac_clean == target_clean || mac_clean.ends_with(&target_clean) || target_clean.ends_with(&mac_clean) {
        return true;
    }

    // 2. Check if target serial ID matches the first 4 bytes of payload (hex string)
    if payload.len() >= 4 {
        let id_le = format!("{:02X}{:02X}{:02X}{:02X}", payload[3], payload[2], payload[1], payload[0]);
        let id_be = format!("{:02X}{:02X}{:02X}{:02X}", payload[0], payload[1], payload[2], payload[3]);
        if id_le.contains(&target_clean) || id_be.contains(&target_clean) {
            return true;
        }
    }

    false
}

/// Decodes tire pressure in bar from common BLE TPMS manufacturer packets:
/// - Sysgration / Jansite / Universal BLE TPMS (32-bit & 16-bit kPa)
/// - Xiaomi / 70mai BLE TPMS
/// - Scaled bar bytes
pub fn decode_tpms_pressure(payload: &[u8]) -> Option<f32> {
    if payload.len() < 4 {
        return None;
    }

    // 1. Sysgration 32-bit: pressure at offset 4..8
    if payload.len() >= 8 {
        let p_raw = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]) as f32;
        // In Pascals * 100 or kPa * 1000
        let bar_100k = p_raw / 100_000.0;
        if (0.5..=5.0).contains(&bar_100k) {
            return Some((bar_100k * 10.0).round() / 10.0);
        }
        // Direct kPa
        let bar_100 = p_raw / 100.0;
        if (0.5..=5.0).contains(&bar_100) {
            return Some((bar_100 * 10.0).round() / 10.0);
        }
    }

    // 2. 16-bit kPa or PSI across common offsets (2..4, 4..6, 6..8)
    for &offset in &[2, 4, 6] {
        if offset + 2 <= payload.len() {
            let kpa_u16 = u16::from_le_bytes([payload[offset], payload[offset + 1]]) as f32;
            if (100.0..=500.0).contains(&kpa_u16) {
                return Some(((kpa_u16 / 100.0) * 10.0).round() / 10.0);
            }
            if (150.0..=600.0).contains(&kpa_u16) {
                let bar = (kpa_u16 / 10.0) / 14.5038;
                if (0.5..=5.0).contains(&bar) {
                    return Some((bar * 10.0).round() / 10.0);
                }
            }
        }
    }

    // 3. Scaled bar byte (e.g. 24 = 2.4 bar)
    for &offset in &[3, 4, 5] {
        if offset < payload.len() {
            let b = payload[offset];
            if (15..=45).contains(&b) {
                return Some(b as f32 / 10.0);
            }
        }
    }

    // 4. Text payload fallback (e.g. "2.4" or "P:2.4")
    if let Ok(text) = std::str::from_utf8(payload) {
        let trimmed = text.trim();
        for token in trimmed.split(|c| c == ' ' || c == ',' || c == ';') {
            let token = token.trim().trim_start_matches(|c| c == 'P' || c == 'p' || c == ':');
            if let Ok(val) = token.parse::<f32>() {
                if (0.5..=5.0).contains(&val) {
                    return Some((val * 10.0).round() / 10.0);
                }
            }
        }
    }

    None
}
