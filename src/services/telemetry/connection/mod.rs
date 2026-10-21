//! Bluetooth adapter and companion app connection probes.

use crate::models::VehicleState;
use std::path::Path;

const APP_CONNECTION_MARKER: &str = "/dev/shm/icarmeter_app_connected";
const APP_CONNECTION_TIMEOUT_SECS: u64 = 15;

pub(super) fn update_connection_state(state: &mut VehicleState) {
    state.bt_active = bluetooth_adapter_is_active();
    state.bt_device_name = read_app_connection();
}

fn bluetooth_adapter_is_active() -> bool {
    Path::new("/sys/class/bluetooth/hci0").exists()
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
        None => "Phone Connected".to_string(),
    })
}
