// Telemetry service coordinator for GPS and Bluetooth sensors

pub mod bt_log;
mod connection;
pub mod gps;
pub mod tpms;

use crate::configs::app::AppConfig;
use crate::models::bluetooth::state::BluetoothSharedState;
use crate::models::vehicle::state::VehicleState;

pub use bt_log::BtLogReader;
pub use gps::GpsReceiver;
pub use tpms::BleTpmsReceiver;

pub struct TelemetryService {
    gps: Option<GpsReceiver>,
    sensors: Option<BleTpmsReceiver>,
    bt_state: Option<BluetoothSharedState>,
    bt_log_reader: BtLogReader,
}

impl TelemetryService {
    pub fn new(config: &AppConfig, bt_state: Option<BluetoothSharedState>) -> Self {
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

        Self {
            gps,
            sensors,
            bt_state,
            bt_log_reader: BtLogReader::new(),
        }
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

    pub fn update(&mut self, state: &mut VehicleState) {
        // 1. Poll hardware GPS first; fall back to mobile companion GPS via Bluetooth
        let mut has_hardware_fix = false;
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
            has_hardware_fix = gps.is_connected();
            state.gps_connected = has_hardware_fix;
        }

        if !has_hardware_fix {
            if let Some(phone) = self.bt_state.as_ref().and_then(|b| b.get_phone_gps()) {
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
            } else if self.gps.is_none() {
                state.gps_connected = false;
            }
        }

        // 2. Connection indicators and hardware adapter state
        connection::update_connection_state(state, self.bt_state.as_ref());

        // 3. BLE TPMS sensor polling
        if let Some(ref mut sensors) = self.sensors {
            sensors.poll(state);
        }

        // 4. Poll Bluetooth logs when view is enabled
        if state.show_bt_log {
            state.bt_logs = self.bt_log_reader.poll(30).to_vec();
        }
    }
}
