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

struct PhoneTelemetry {
    spd: Option<f32>,
    lat: Option<f64>,
    lon: Option<f64>,
    alt: Option<f32>,
    heading: Option<f32>,
}

fn poll_phone_telemetry() -> Option<PhoneTelemetry> {
    let metadata = std::fs::metadata(PHONE_TELEMETRY_MARKER).ok()?;
    let modified = metadata.modified().ok()?;
    if modified
        .elapsed()
        .map_or(true, |el| el >= PHONE_TELEMETRY_TIMEOUT)
    {
        return None;
    }

    let content = std::fs::read_to_string(PHONE_TELEMETRY_MARKER).ok()?;
    let mut tel = PhoneTelemetry {
        spd: None,
        lat: None,
        lon: None,
        alt: None,
        heading: None,
    };

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("spd=") {
            tel.spd = val.trim().parse::<f32>().ok();
        } else if let Some(val) = line.strip_prefix("lat=") {
            tel.lat = val.trim().parse::<f64>().ok();
        } else if let Some(val) = line.strip_prefix("lon=") {
            tel.lon = val.trim().parse::<f64>().ok();
        } else if let Some(val) = line.strip_prefix("alt=") {
            tel.alt = val.trim().parse::<f32>().ok();
        } else if let Some(val) = line.strip_prefix("heading=") {
            tel.heading = val.trim().parse::<f32>().ok();
        }
    }

    Some(tel)
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
            if let Some(update) = gps.poll_update() {
                if let Some(speed) = update.speed_kmh {
                    state.current_speed = speed.max(0.0);
                }
                if let Some(lat) = update.latitude {
                    state.gps_latitude = lat;
                }
                if let Some(lon) = update.longitude {
                    state.gps_longitude = lon;
                }
                if let Some(hdg) = update.heading {
                    state.gps_heading = hdg;
                }
            }
            state.gps_connected = gps.is_connected();
        } else if let Some(phone) = poll_phone_telemetry() {
            if let Some(speed) = phone.spd {
                state.current_speed = speed.max(0.0);
            }
            if let Some(lat) = phone.lat {
                state.gps_latitude = lat;
            }
            if let Some(lon) = phone.lon {
                state.gps_longitude = lon;
            }
            if let Some(alt) = phone.alt {
                state.gps_altitude = alt;
            }
            if let Some(hdg) = phone.heading {
                state.gps_heading = hdg;
            }
            state.gps_connected = true;
        } else {
            state.gps_connected = false;
        }

        // Connection indicators are independent from sensor polling.
        connection::update_connection_state(state);
        state.update_trip(dt);

        if let Some(ref mut sensors) = self.sensors {
            sensors.poll(state);
        }
    }
}
