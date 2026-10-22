// Telemetry module for Tire Pressure Monitoring System (TPMS)

use crate::configs::app::TpmsConfig;
use crate::models::vehicle::state::VehicleState;

pub struct TpmsReceiver {
    #[allow(dead_code)]
    config: TpmsConfig,
}

impl TpmsReceiver {
    pub fn new(config: &TpmsConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn update_config(&mut self, new_config: &TpmsConfig) {
        self.config = new_config.clone();
    }

    /// Non-blocking update called every frame on the telemetry loop
    pub fn poll(&mut self, _state: &mut VehicleState) {
        // Reserved for future TPMS implementation (e.g. CAN bus, serial, direct sensors)
    }
}