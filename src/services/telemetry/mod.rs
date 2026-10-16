// Telemetry service coordinator for GPS and Bluetooth sensors

pub mod gps;
pub mod sensors;

use crate::configs::AppConfig;
use crate::models::VehicleState;
pub use gps::GpsReceiver;
pub use sensors::SensorReceiver;

pub struct TelemetryService {
    gps: Option<GpsReceiver>,
    sensors: Option<SensorReceiver>,
}

impl TelemetryService {
    pub fn new(config: &AppConfig) -> Self {
        let gps = if config.gps.enabled {
            Some(GpsReceiver::new(&config.gps.device, config.gps.baud_rate))
        } else {
            None
        };

        let sensors = if config.sensors.enabled {
            Some(SensorReceiver::new(&config.sensors.device, config.sensors.baud_rate))
        } else {
            None
        };

        Self { gps, sensors }
    }

    pub fn update(&mut self, state: &mut VehicleState, dt: f32) {
        // 1. Poll GPS for live speed
        if let Some(ref mut gps) = self.gps {
            if let Some(speed) = gps.poll_speed() {
                state.current_speed = speed.max(0.0);
            }
        }

        // 2. Update real trip telemetry (odometer, driving time, avg speed, max speed)
        state.update_trip(dt);

        // 3. Poll Bluetooth sensors for live TPMS & commands
        if let Some(ref mut sensors) = self.sensors {
            sensors.poll(state);
        }
    }
}
