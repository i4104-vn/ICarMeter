//! Bluetooth adapter and companion app connection probes.

use crate::models::bluetooth::state::BluetoothSharedState;
use crate::models::vehicle::state::VehicleState;
use crate::services::telemetry::bt_log::log_bt;
use std::path::Path;

pub(super) fn update_connection_state(
    state: &mut VehicleState,
    bt_state: Option<&BluetoothSharedState>,
) {
    let was_active = state.bt_active;
    state.bt_active = bluetooth_adapter_is_active();
    if was_active != state.bt_active {
        log_bt(
            "BT-PROBE",
            &format!(
                "Bluetooth adapter hci0 state changed: {}",
                if state.bt_active {
                    "Active"
                } else {
                    "Inactive/Missing"
                }
            ),
        );
    }

    let connected = bt_state.and_then(|b| b.connected_peer());
    if state.bt_device_name != connected {
        match &connected {
            Some(dev) => log_bt("BT-SPP", &format!("Companion phone connected: {}", dev)),
            None => {
                if state.bt_device_name.is_some() {
                    log_bt("BT-SPP", "Companion phone disconnected");
                }
            }
        }
        state.bt_device_name = connected;
    }
}

fn bluetooth_adapter_is_active() -> bool {
    Path::new("/sys/class/bluetooth/hci0").exists()
}
