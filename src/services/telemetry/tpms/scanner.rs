// Background Bluetooth LE scanner loop capturing advertising packets

use super::decoder::{decode_tpms_pressure, decode_tpms_temperature, matches_sensor_id};
use super::hci_socket::{EVT_LE_ADVERTISING_REPORT, EVT_LE_META_EVENT, HCI_EVENT_PKT, HciSocket};
use crate::configs::app::BleTpmsConfig;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Default, Clone, Copy)]
pub struct SharedTirePressures {
    pub fl: f32,
    pub fr: f32,
    pub rl: f32,
    pub rr: f32,
    pub fl_temp: f32,
    pub fr_temp: f32,
    pub rl_temp: f32,
    pub rr_temp: f32,
}

/// Continuous BLE advertising packet receiver loop running in background thread
pub fn run_ble_scanner_loop(
    config: BleTpmsConfig,
    pressures: Arc<Mutex<SharedTirePressures>>,
    expression: Arc<Mutex<Option<String>>>,
) {
    let mut discovered_sensors = HashSet::new();

    loop {
        let socket = match HciSocket::open(&config.adapter) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "[TPMS-BLE] Could not open raw Bluetooth socket on {}: {} (retrying in 5s)",
                    config.adapter, e
                );
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        eprintln!(
            "[TPMS-BLE] Bluetooth LE scanner actively listening on {}...",
            config.adapter
        );

        let mut read_buf = [0u8; 1024];

        loop {
            let bytes_read = socket.read(&mut read_buf);
            if bytes_read <= 0 {
                break;
            }

            let len = bytes_read as usize;
            if len < 14 {
                continue;
            }

            // Verify HCI Event Packet -> LE Meta Event -> LE Advertising Report
            if read_buf[0] != HCI_EVENT_PKT
                || read_buf[1] != EVT_LE_META_EVENT
                || read_buf[3] != EVT_LE_ADVERTISING_REPORT
            {
                continue;
            }

            // Extract reverse MAC address (6 bytes from offset 7 to 12)
            let mac_str = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                read_buf[12], read_buf[11], read_buf[10], read_buf[9], read_buf[8], read_buf[7]
            );

            let data_len = (read_buf[13] as usize).min(len.saturating_sub(14));
            let data = &read_buf[14..14 + data_len];

            // Parse AD elements
            let mut offset = 0;
            while offset + 1 < data.len() {
                let ad_len = data[offset] as usize;
                if ad_len == 0 || offset + 1 + ad_len > data.len() {
                    break;
                }

                let ad_type = data[offset + 1];
                let payload = &data[offset + 2..offset + 1 + ad_len];

                // 1. Check for complete local name (EMO trigger or TPMS name)
                if (ad_type == 0x09 || ad_type == 0x08)
                    && !payload.is_empty()
                    && let Ok(name) = std::str::from_utf8(payload)
                    && let Some(emo_name) = name.strip_prefix("EMO:")
                    && let Ok(mut expr) = expression.lock()
                {
                    *expr = Some(emo_name.trim().to_string());
                }

                // 2. Check for Manufacturer Specific Data (0xFF) or Service Data (0x16)
                if (ad_type == 0xFF || ad_type == 0x16)
                    && let Some(pressure) = decode_tpms_pressure(payload)
                {
                    let temp = decode_tpms_temperature(payload).unwrap_or(0.0);
                    let mut matched = false;

                    if matches_sensor_id(&config.fl_sensor, &mac_str, payload) {
                        if let Ok(mut p) = pressures.lock() {
                            p.fl = pressure;
                            if temp != 0.0 {
                                p.fl_temp = temp;
                            }
                        }
                        matched = true;
                    } else if matches_sensor_id(&config.fr_sensor, &mac_str, payload) {
                        if let Ok(mut p) = pressures.lock() {
                            p.fr = pressure;
                            if temp != 0.0 {
                                p.fr_temp = temp;
                            }
                        }
                        matched = true;
                    } else if matches_sensor_id(&config.rl_sensor, &mac_str, payload) {
                        if let Ok(mut p) = pressures.lock() {
                            p.rl = pressure;
                            if temp != 0.0 {
                                p.rl_temp = temp;
                            }
                        }
                        matched = true;
                    } else if matches_sensor_id(&config.rr_sensor, &mac_str, payload) {
                        if let Ok(mut p) = pressures.lock() {
                            p.rr = pressure;
                            if temp != 0.0 {
                                p.rr_temp = temp;
                            }
                        }
                        matched = true;
                    }

                    // Discovery notification for unmatched sensors (helps user find their sensor MACs)
                    if !matched && discovered_sensors.insert(mac_str.clone()) {
                        eprintln!(
                            "[TPMS-BLE] Discovered sensor MAC: {} (Pressure: {:.1} bar, Temp: {:.0}°C)",
                            mac_str, pressure, temp
                        );
                    }
                }

                offset += 1 + ad_len;
            }
        }

        thread::sleep(Duration::from_secs(2));
    }
}
