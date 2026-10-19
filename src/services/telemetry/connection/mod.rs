//! Bluetooth adapter and companion-app connection probes.

use crate::models::VehicleState;
use std::path::Path;

const APP_CONNECTION_MARKER: &str = "/dev/shm/icarmeter_app_connected";
const WS_EVENT_MARKER: &str = "/dev/shm/icarmeter_ws_event";
const HOTSPOT_MARKER: &str = "/dev/shm/icarmeter_hotspot_active";
const APP_CONNECTION_TIMEOUT_SECS: u64 = 15;

pub(super) fn update_connection_state(state: &mut VehicleState) {
    state.bt_active = bluetooth_adapter_is_active();
    state.wifi_active = wifi_hotspot_is_active();

    // Read dedicated 1-line WebSocket event log
    state.ws_event_log = read_ws_event();

    // App connection status: active when app marker is fresh or ws event is alive
    state.app_connected = read_app_connection().is_some()
        || state.ws_event_log.as_deref().map_or(false, |s| {
            !s.contains("Disconnected") && !s.contains("Offline") && !s.contains("Idle")
        });

    // Bluetooth is hardware/BLE only — isolated from WebSocket
    state.bt_device_name = None;
}

fn wifi_hotspot_is_active() -> bool {
    Path::new(HOTSPOT_MARKER).exists()
        || Path::new("/sys/class/net/wlan0").exists()
        || cfg!(target_os = "windows")
}

fn bluetooth_adapter_is_active() -> bool {
    Path::new("/sys/class/bluetooth/hci0").exists() || cfg!(target_os = "windows")
}

fn read_ws_event() -> Option<String> {
    let metadata = std::fs::metadata(WS_EVENT_MARKER).ok()?;
    let modified = metadata.modified().ok()?;
    // If no events received for > 30 seconds, mark as standby
    if modified.elapsed().map_or(true, |elapsed| elapsed.as_secs() >= 30) {
        return Some("Standby".to_string());
    }

    let content = std::fs::read_to_string(WS_EVENT_MARKER).ok()?;
    let line = content.lines().next()?.trim();
    if !line.is_empty() {
        Some(clean_event_name(line))
    } else {
        None
    }
}

fn clean_event_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("connected") || trimmed.contains("Connected") {
        "Connected".to_string()
    } else if trimmed.eq_ignore_ascii_case("telemetry")
        || trimmed.contains("Telemetry")
        || trimmed.contains("gps")
    {
        "Telemetry".to_string()
    } else if trimmed.contains("Saved") || trimmed.contains("set_config") {
        "Config Saved".to_string()
    } else if trimmed.contains("Sync") || trimmed.contains("get_config") {
        "Config Synced".to_string()
    } else if trimmed.contains("Disconnect") {
        "Disconnected".to_string()
    } else if trimmed.contains("Standby") || trimmed.contains("Idle") {
        "Standby".to_string()
    } else {
        let words: Vec<&str> = trimmed.split_whitespace().take(2).collect();
        let joined = words.join(" ");
        if joined.len() > 14 {
            joined[..14].to_string()
        } else {
            joined
        }
    }
}

fn read_app_connection() -> Option<String> {
    let metadata = std::fs::metadata(APP_CONNECTION_MARKER).ok()?;
    let modified = metadata.modified().ok()?;
    if modified.elapsed().map_or(true, |elapsed| {
        elapsed.as_secs() >= APP_CONNECTION_TIMEOUT_SECS
    }) {
        return None;
    }

    let content = std::fs::read_to_string(APP_CONNECTION_MARKER).ok();
    Some(match content.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() && name != "1" => name.to_string(),
        Some(_) => "Phone Connected".to_string(),
        None => "Connected".to_string(),
    })
}
