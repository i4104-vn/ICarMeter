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

    pub fn update(&mut self, state: &mut VehicleState, dt: f32) {
        // 1. Poll GPS for live speed and connection state
        if let Some(ref mut gps) = self.gps {
            if let Some(speed) = gps.poll_speed() {
                state.current_speed = speed.max(0.0);
            }
            // A serial device can be open while the receiver has no valid
            // satellite fix. Only expose GPS as active after valid NMEA data.
            state.gps_connected = gps.is_connected();
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
