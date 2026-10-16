// Wireless Bluetooth Low Energy (BLE) 4-Tire TPMS receiver for Raspberry Pi OS

use crate::configs::hardware::BleTpmsConfig;
use crate::models::VehicleState;
use std::collections::HashSet;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const AF_BLUETOOTH: libc::c_int = 31;
const BTPROTO_HCI: libc::c_int = 1;
const SOL_HCI: libc::c_int = 0;
const HCI_FILTER: libc::c_int = 2;
const HCI_EVENT_PKT: u8 = 0x04;
const EVT_LE_META_EVENT: u8 = 0x3E;
const EVT_LE_ADVERTISING_REPORT: u8 = 0x02;

#[repr(C)]
struct SockAddrHci {
    hci_family: libc::sa_family_t,
    hci_dev: u16,
    hci_channel: u16,
}

#[repr(C)]
struct HciFilter {
    type_mask: u32,
    event_mask: [u32; 2],
    opcode: u16,
}

#[derive(Default, Clone, Copy)]
struct SharedTires {
    fl: f32,
    fr: f32,
    rl: f32,
    rr: f32,
}

pub struct SensorReceiver {
    pressures: Arc<Mutex<SharedTires>>,
    expression: Arc<Mutex<Option<String>>>,
}

impl SensorReceiver {
    pub fn new(config: &BleTpmsConfig) -> Self {
        let pressures = Arc::new(Mutex::new(SharedTires::default()));
        let expression = Arc::new(Mutex::new(None));

        let thread_pressures = Arc::clone(&pressures);
        let thread_expression = Arc::clone(&expression);
        let config_clone = config.clone();

        thread::Builder::new()
            .name("bt-tpms-scanner".to_string())
            .spawn(move || {
                run_ble_scanner(config_clone, thread_pressures, thread_expression);
            })
            .expect("Failed to spawn Bluetooth TPMS scanner thread");

        Self {
            pressures,
            expression,
        }
    }

    /// Non-blocking update called every frame on the render loop (60 FPS)
    pub fn poll(&mut self, state: &mut VehicleState) {
        if let Ok(guard) = self.pressures.try_lock() {
            if guard.fl > 0.0 {
                state.tires.fl = guard.fl;
            }
            if guard.fr > 0.0 {
                state.tires.fr = guard.fr;
            }
            if guard.rl > 0.0 {
                state.tires.rl = guard.rl;
            }
            if guard.rr > 0.0 {
                state.tires.rr = guard.rr;
            }
        }

        if let Ok(mut expr_guard) = self.expression.try_lock() {
            if let Some(ref name) = *expr_guard {
                state.set_expression_name(name);
            }
            *expr_guard = None;
        }
    }
}

/// Continuous BLE advertising packet receiver loop running in background thread
fn run_ble_scanner(
    config: BleTpmsConfig,
    pressures: Arc<Mutex<SharedTires>>,
    expression: Arc<Mutex<Option<String>>>,
) {
    let mut discovered_sensors = HashSet::new();

    loop {
        // 1. Ensure Bluetooth adapter is powered UP
        let _ = Command::new("hciconfig")
            .args([&config.adapter, "up"])
            .output();

        // 2. Open raw HCI Bluetooth socket
        let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_RAW, BTPROTO_HCI) };
        if fd < 0 {
            let err = std::io::Error::last_os_error();
            eprintln!(
                "[TPMS-BLE] Could not open raw Bluetooth socket on {}: {} (retrying in 5s)",
                config.adapter, err
            );
            thread::sleep(Duration::from_secs(5));
            continue;
        }

        // 3. Bind socket to hci0
        let sa = SockAddrHci {
            hci_family: AF_BLUETOOTH as libc::sa_family_t,
            hci_dev: 0, // hci0
            hci_channel: 0,
        };

        let bind_res = unsafe {
            libc::bind(
                fd,
                &sa as *const _ as *const libc::sockaddr,
                std::mem::size_of::<SockAddrHci>() as libc::socklen_t,
            )
        };

        if bind_res < 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("[TPMS-BLE] Bind failed for {}: {}", config.adapter, err);
            unsafe { libc::close(fd) };
            thread::sleep(Duration::from_secs(5));
            continue;
        }

        // 4. Set filter for HCI Event packets (LE meta advertising report)
        let filter = HciFilter {
            type_mask: 1 << HCI_EVENT_PKT, // Listen for HCI Event packets
            event_mask: [0xFFFF_FFFF, 0xFFFF_FFFF], // All event codes
            opcode: 0,
        };

        unsafe {
            libc::setsockopt(
                fd,
                SOL_HCI,
                HCI_FILTER,
                &filter as *const _ as *const libc::c_void,
                std::mem::size_of::<HciFilter>() as libc::socklen_t,
            );
        }

        // 5. Enable LE Scan via HCI commands
        // LE Set Scan Parameters (passive scan, 10ms interval/window)
        let scan_params_cmd = [
            0x01, 0x0B, 0x20, 7, // Header: OCF 0x000B, OGF 0x08, Len 7
            0x00, // Passive scan
            0x10, 0x00, // Interval
            0x10, 0x00, // Window
            0x00, // Public address
            0x00, // All packets
        ];
        unsafe {
            libc::write(
                fd,
                scan_params_cmd.as_ptr() as *const libc::c_void,
                scan_params_cmd.len(),
            );
        }

        // LE Set Scan Enable (enable = 1, filter_duplicates = 0)
        let scan_enable_cmd = [
            0x01, 0x0C, 0x20, 2, // Header: OCF 0x000C, OGF 0x08, Len 2
            0x01, // Scan enable
            0x00, // Filter duplicates: 0
        ];
        unsafe {
            libc::write(
                fd,
                scan_enable_cmd.as_ptr() as *const libc::c_void,
                scan_enable_cmd.len(),
            );
        }

        println!(
            "[TPMS-BLE] Bluetooth LE scanner actively listening on {}...",
            config.adapter
        );

        let mut read_buf = [0u8; 1024];

        while let Ok(bytes_read) =
            unsafe { Ok::<isize, std::io::Error>(libc::read(fd, read_buf.as_mut_ptr() as *mut libc::c_void, read_buf.len())) }
        {
            if bytes_read <= 0 {
                break;
            }

            let len = bytes_read as usize;
            if len < 14 {
                continue;
            }

            // Verify HCI Event Packet -> LE Meta Event -> LE Advertising Report
            if read_buf[0] != HCI_EVENT_PKT || read_buf[1] != EVT_LE_META_EVENT || read_buf[3] != EVT_LE_ADVERTISING_REPORT {
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
                if (ad_type == 0x09 || ad_type == 0x08) && !payload.is_empty() {
                    if let Ok(name) = std::str::from_utf8(payload) {
                        if let Some(emo_name) = name.strip_prefix("EMO:") {
                            if let Ok(mut expr) = expression.lock() {
                                *expr = Some(emo_name.trim().to_string());
                            }
                        }
                    }
                }

                // 2. Check for Manufacturer Specific Data (0xFF) or Service Data (0x16)
                if ad_type == 0xFF || ad_type == 0x16 {
                    if let Some(pressure) = decode_ble_tpms_pressure(payload) {
                        let mut matched = false;

                        if matches_sensor_id(&config.fl_sensor, &mac_str, payload) {
                            if let Ok(mut p) = pressures.lock() {
                                p.fl = pressure;
                            }
                            matched = true;
                        } else if matches_sensor_id(&config.fr_sensor, &mac_str, payload) {
                            if let Ok(mut p) = pressures.lock() {
                                p.fr = pressure;
                            }
                            matched = true;
                        } else if matches_sensor_id(&config.rl_sensor, &mac_str, payload) {
                            if let Ok(mut p) = pressures.lock() {
                                p.rl = pressure;
                            }
                            matched = true;
                        } else if matches_sensor_id(&config.rr_sensor, &mac_str, payload) {
                            if let Ok(mut p) = pressures.lock() {
                                p.rr = pressure;
                            }
                            matched = true;
                        }

                        // Discovery notification for unmatched sensors (helps user find their sensor MACs)
                        if !matched && discovered_sensors.insert(mac_str.clone()) {
                            println!(
                                "[TPMS-BLE] Discovered sensor MAC: {} (Pressure: {:.1} bar)",
                                mac_str, pressure
                            );
                        }
                    }
                }

                offset += 1 + ad_len;
            }
        }

        unsafe { libc::close(fd) };
        thread::sleep(Duration::from_secs(2));
    }
}

/// Matches target sensor ID against MAC address or payload serial bytes
fn matches_sensor_id(target: &str, mac: &str, payload: &[u8]) -> bool {
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
pub fn decode_ble_tpms_pressure(payload: &[u8]) -> Option<f32> {
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
