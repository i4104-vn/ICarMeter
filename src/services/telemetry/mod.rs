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
    if modified
        .elapsed()
        .map_or(true, |el| el >= PHONE_TELEMETRY_TIMEOUT)
    {
        return None;
    }

    std::fs::read_to_string(PHONE_TELEMETRY_MARKER)
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("spd=")
                .and_then(|value| value.trim().parse::<f32>().ok())
        })
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
        match config.gps.enabled {
            true => {
                let needs_reopen = self.gps.as_ref().is_none_or(|gps| {
                    !gps.matches_config(&config.gps.device, config.gps.baud_rate)
                });
                if needs_reopen {
                    self.gps = Some(GpsReceiver::new(&config.gps.device, config.gps.baud_rate));
                }
            }
            false => self.gps = None,
        }

        match config.tpms.enabled {
            true => match self.sensors.as_mut() {
                Some(sensors) => sensors.update_config(&config.tpms),
                None => self.sensors = Some(BleTpmsReceiver::new(&config.tpms)),
            },
            false => self.sensors = None,
        }
    }

    pub fn update(&mut self, state: &mut VehicleState, dt: f32) {
        // Poll hardware GPS first, then fall back to the companion app marker.
        if let Some(gps) = self.gps.as_mut() {
            if let Some(speed) = gps.poll_speed() {
                state.current_speed = speed.max(0.0);
            }
            state.gps_connected = gps.is_connected();
        } else {
            match poll_phone_telemetry() {
                Some(speed) => {
                    state.current_speed = speed.max(0.0);
                    state.gps_connected = true;
                }
                None => state.gps_connected = false,
            }
        }

        // Connection indicators are independent from sensor polling.
        connection::update_connection_state(state);
        state.update_trip(dt);

        if let Some(ref mut sensors) = self.sensors {
            sensors.poll(state);
        }
    }
}
