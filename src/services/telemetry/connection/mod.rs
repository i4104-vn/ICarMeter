//! Bluetooth adapter and companion-app connection probes.

mod event_name;

use crate::models::VehicleState;
use event_name::clean_event_name;
use std::path::Path;

const APP_CONNECTION_MARKER: &str = "/dev/shm/icarmeter_app_connected";
const WS_EVENT_MARKER: &str = "/dev/shm/icarmeter_ws_event";
const HOTSPOT_MARKER: &str = "/dev/shm/icarmeter_hotspot_active";
const APP_CONNECTION_TIMEOUT_SECS: u64 = 15;
const WS_EVENT_TIMEOUT_SECS: u64 = 3;

pub(super) fn update_connection_state(state: &mut VehicleState) {
    state.bt_active = bluetooth_adapter_is_active();
    state.wifi_active = wifi_hotspot_is_active();

    // Read dedicated 1-line WebSocket event log
    state.ws_event_log = read_ws_event();

    // App connection status: active when app marker is fresh or ws event is alive
    state.app_connected = read_app_connection().is_some()
        || state.ws_event_log.as_deref().is_some_and(|event| {
            !event.contains("Disconnected") && !event.contains("Offline") && !event.contains("Idle")
        });

    // Bluetooth is hardware/BLE only — isolated from WebSocket
    state.bt_device_name = None;
}

fn wifi_hotspot_is_active() -> bool {
    Path::new(HOTSPOT_MARKER).exists() || Path::new("/sys/class/net/wlan0").exists()
}

fn bluetooth_adapter_is_active() -> bool {
    Path::new("/sys/class/bluetooth/hci0").exists()
}

fn read_ws_event() -> Option<String> {
    let metadata = std::fs::metadata(WS_EVENT_MARKER).ok()?;
    let modified = metadata.modified().ok()?;
    let elapsed = match modified.elapsed() {
        Ok(d) => d.as_secs_f32(),
        Err(e) => {
            let future_diff = e.duration().as_secs_f32();
            if future_diff < 2.0 { 0.0 } else { 100.0 }
        }
    };
    // If no event received for the timeout, the event has ended.
    if elapsed >= WS_EVENT_TIMEOUT_SECS as f32 {
        return None;
    }

    let content = std::fs::read_to_string(WS_EVENT_MARKER).ok()?;
    let line = content.lines().next()?.trim();
    if line.is_empty() || line.eq_ignore_ascii_case("standby") || line.eq_ignore_ascii_case("idle")
    {
        return None;
    }

    let cleaned = clean_event_name(line);
    if cleaned == "Standby" {
        None
    } else {
        Some(cleaned)
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
