// Background Bluetooth LE scanner loop capturing advertising packets

use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use super::decoder::{decode_tpms_pressure, decode_tpms_temperature, matches_sensor_id};
use super::hci_socket::HciSocket;
use crate::configs::app::BleTpmsConfig;
use crate::models::bluetooth::socket::{
    EVT_LE_ADVERTISING_REPORT, EVT_LE_META_EVENT, HCI_EVENT_PKT,
};
use crate::models::tpms::shared::SharedTirePressures;
use crate::services::telemetry::bt_log::log_bt;

/// Continuous BLE advertising packet receiver loop running in background thread
pub fn run_ble_scanner_loop(
    config: Arc<RwLock<BleTpmsConfig>>,
    pressures: Arc<Mutex<SharedTirePressures>>,
    expression: Arc<Mutex<Option<String>>>,
) {
    let mut discovered_sensors = HashSet::new();

    loop {
        let adapter = match config.read() {
            Ok(c) => c.adapter.clone(),
            Err(_) => "hci0".to_string(),
        };

        let socket = match HciSocket::open(&adapter) {
            Ok(s) => s,
            Err(e) => {
                let err_msg = format!(
                    "Could not open raw Bluetooth socket on {}: {} (retrying in 5s)",
                    adapter, e
                );
                eprintln!("[TPMS-BLE] {}", err_msg);
                log_bt("TPMS-BLE", &err_msg);
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        let ready_msg = format!("Bluetooth LE scanner actively listening on {}...", adapter);
        eprintln!("[TPMS-BLE] {}", ready_msg);
        log_bt("TPMS-BLE", &ready_msg);

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

                    let sensor_targets = match config.read() {
                        Ok(c) => [
                            c.fl_sensor.clone(),
                            c.fr_sensor.clone(),
                            c.rl_sensor.clone(),
                            c.rr_sensor.clone(),
                        ],
                        Err(_) => continue,
                    };

                    if let Some((sensor_index, _)) = sensor_targets
                        .iter()
                        .enumerate()
                        .find(|(_, target)| matches_sensor_id(target, &mac_str, payload))
                    {
                        update_pressure(&pressures, sensor_index, pressure, temp);
                        matched = true;
                    }

                    // Discovery notification for unmatched sensors (helps user find their sensor MACs)
                    if !matched && discovered_sensors.insert(mac_str.clone()) {
                        let disc_msg = format!(
                            "Discovered sensor MAC: {} (Pressure: {:.1} bar, Temp: {:.0}°C)",
                            mac_str, pressure, temp
                        );
                        eprintln!("[TPMS-BLE] {}", disc_msg);
                        log_bt("TPMS-BLE", &disc_msg);
                    }
                }

                offset += 1 + ad_len;
            }
        }

        thread::sleep(Duration::from_secs(2));
    }
}

fn update_pressure(
    pressures: &Arc<Mutex<SharedTirePressures>>,
    sensor_index: usize,
    pressure: f32,
    temperature: f32,
) {
    let Ok(mut pressures) = pressures.lock() else {
        return;
    };

    match sensor_index {
        0 => {
            pressures.fl = pressure;
            if temperature != 0.0 {
                pressures.fl_temp = temperature;
            }
        }
        1 => {
            pressures.fr = pressure;
            if temperature != 0.0 {
                pressures.fr_temp = temperature;
            }
        }
        2 => {
            pressures.rl = pressure;
            if temperature != 0.0 {
                pressures.rl_temp = temperature;
            }
        }
        3 => {
            pressures.rr = pressure;
            if temperature != 0.0 {
                pressures.rr_temp = temperature;
            }
        }
        _ => {}
    }
}
