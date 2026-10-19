// Telemetry service coordinator for GPS and Bluetooth sensors

mod connection;
pub mod gps;
pub mod tpms;

use crate::configs::AppConfig;
use crate::models::VehicleState;

pub use gps::GpsReceiver;
pub use tpms::BleTpmsReceiver;

pub struct TelemetryService {
    gps: Option<GpsReceiver>,
    sensors: Option<BleTpmsReceiver>,
}

const PHONE_TELEMETRY_MARKER: &str = "/dev/shm/icarmeter_phone_telemetry";
const PHONE_TELEMETRY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

fn poll_phone_telemetry() -> Option<f32> {
    let metadata = std::fs::metadata(PHONE_TELEMETRY_MARKER).ok()?;
    let modified = metadata.modified().ok()?;
    if modified.elapsed().map_or(true, |el| el >= PHONE_TELEMETRY_TIMEOUT) {
        return None;
    }

    let content = std::fs::read_to_string(PHONE_TELEMETRY_MARKER).ok()?;
    for line in content.lines() {
        if let Some(val_str) = line.strip_prefix("spd=") {
            if let Ok(spd) = val_str.trim().parse::<f32>() {
                return Some(spd);
            }
        }
    }
    None
}

impl TelemetryService {
    pub fn new(config: &AppConfig) -> Self {
        let gps = if config.gps.enabled {
            Some(GpsReceiver::new(&config.gps.device, config.gps.baud_rate))
        } else {
            None
        };

        let sensors = if config.tpms.enabled {
            Some(BleTpmsReceiver::new(&config.tpms))
        } else {
            None
        };

        Self { gps, sensors }
    }

    pub fn reload_config(&mut self, config: &AppConfig) {
        if config.gps.enabled {
            if self.gps.is_none() {
                self.gps = Some(GpsReceiver::new(&config.gps.device, config.gps.baud_rate));
            }
        } else {
            self.gps = None;
        }

        if let Some(ref mut sensors) = self.sensors {
            sensors.update_config(&config.tpms);
        } else if config.tpms.enabled {
            self.sensors = Some(BleTpmsReceiver::new(&config.tpms));
        }
    }

    pub fn update(&mut self, state: &mut VehicleState, dt: f32) {
        // 1. Poll GPS for live speed and connection state (Hardware GPS or Phone Wi-Fi Telemetry)
        if let Some(ref mut gps) = self.gps {
            if let Some(speed) = gps.poll_speed() {
                state.current_speed = speed.max(0.0);
            }
            state.gps_connected = gps.is_connected();
        } else if let Some(phone_speed) = poll_phone_telemetry() {
            state.current_speed = phone_speed.max(0.0);
            state.gps_connected = true;
        } else {
            state.gps_connected = false;
        }

        // 2. Update connection indicators independently from sensor polling.
        connection::update_connection_state(state);

        // 3. Update real trip telemetry (odometer, driving time, avg speed, max speed)
        state.update_trip(dt);

        // 4. Poll wireless Bluetooth TPMS sensors
        if let Some(ref mut sensors) = self.sensors {
            sensors.poll(state);
        }
    }
}
