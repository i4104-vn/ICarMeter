// Telemetry service coordinator running on dedicated background worker thread

pub mod bt_log;
pub mod gps;
pub mod tpms;

use crate::configs::app::AppConfig;
use crate::models::bluetooth::state::BluetoothSharedState;
use crate::models::tpms::pressures::TirePressures;
use crate::models::vehicle::state::VehicleState;
use crate::services::is_app_running;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub use bt_log::BtLogReader;
pub use gps::GpsReceiver;
pub use tpms::BleTpmsReceiver;

#[derive(Clone, Debug, Default)]
pub struct TelemetrySnapshot {
    pub current_speed: Option<f32>,
    pub gps_latitude: f64,
    pub gps_longitude: f64,
    pub gps_heading: f32,
    pub gps_altitude: f32,
    pub gps_connected: bool,
    pub tires: TirePressures,
    pub bt_active: bool,
    pub bt_device_name: Option<String>,
    pub bt_logs: Vec<String>,
}

pub struct TelemetryService {
    snapshot: Arc<Mutex<TelemetrySnapshot>>,
    config_tx: Sender<AppConfig>,
    log_view_enabled: Arc<AtomicBool>,
}

impl TelemetryService {
    pub fn new(config: &AppConfig, bt_state: Option<BluetoothSharedState>) -> Self {
        let snapshot = Arc::new(Mutex::new(TelemetrySnapshot::default()));
        let log_view_enabled = Arc::new(AtomicBool::new(false));
        let (config_tx, config_rx) = mpsc::channel::<AppConfig>();

        let thread_snapshot = Arc::clone(&snapshot);
        let thread_log_view = Arc::clone(&log_view_enabled);
        let initial_config = config.clone();

        thread::Builder::new()
            .name("telemetry-worker".to_string())
            .spawn(move || {
                run_telemetry_loop(
                    initial_config,
                    config_rx,
                    bt_state,
                    thread_snapshot,
                    thread_log_view,
                );
            })
            .expect("Failed to spawn telemetry-worker thread");

        Self {
            snapshot,
            config_tx,
            log_view_enabled,
        }
    }

    pub fn reload_config(&mut self, config: &AppConfig) {
        let _ = self.config_tx.send(config.clone());
    }

    /// Fast, non-blocking state sync called from main UI render thread.
    /// Performs zero I/O, zero syscalls, zero blocking locks.
    pub fn update(&mut self, state: &mut VehicleState) {
        self.log_view_enabled
            .store(state.show_bt_log, Ordering::Relaxed);

        if let Ok(guard) = self.snapshot.lock() {
            if let Some(speed) = guard.current_speed {
                state.current_speed = speed;
            }
            state.gps_latitude = guard.gps_latitude;
            state.gps_longitude = guard.gps_longitude;
            state.gps_heading = guard.gps_heading;
            state.gps_altitude = guard.gps_altitude;
            state.gps_connected = guard.gps_connected;
            state.tires = guard.tires.clone();
            state.bt_active = guard.bt_active;
            state.bt_device_name = guard.bt_device_name.clone();
            if state.show_bt_log && !guard.bt_logs.is_empty() {
                state.bt_logs = guard.bt_logs.clone();
            }
        }
    }
}

/// Independent telemetry loop handling GPS serial I/O, BLE TPMS scanning,
/// Bluetooth adapter status checking, and log reading off the main thread.
fn run_telemetry_loop(
    config: AppConfig,
    config_rx: Receiver<AppConfig>,
    bt_state: Option<BluetoothSharedState>,
    snapshot: Arc<Mutex<TelemetrySnapshot>>,
    log_view_enabled: Arc<AtomicBool>,
) {
    let mut gps = if config.gps.enabled {
        Some(GpsReceiver::new(&config.gps.device, config.gps.baud_rate))
    } else {
        None
    };

    let mut sensors = if config.tpms.enabled {
        Some(BleTpmsReceiver::new(&config.tpms))
    } else {
        None
    };

    let mut bt_log_reader = BtLogReader::new();
    let mut last_tpms_publish = Instant::now();
    let mut current_tires = TirePressures::default();
    let mut current_speed: Option<f32> = None;
    let mut current_lat = 21.0285;
    let mut current_lon = 105.8542;
    let mut current_heading = 0.0;
    let mut current_altitude = 18.0;
    let mut gps_connected = false;
    let mut bt_active = true;
    let mut bt_device_name: Option<String> = None;

    while is_app_running() {
        // 1. Process dynamic config updates
        while let Ok(new_config) = config_rx.try_recv() {
            match new_config.gps.enabled {
                true => {
                    let needs_reopen = gps.as_ref().is_none_or(|g| {
                        !g.matches_config(&new_config.gps.device, new_config.gps.baud_rate)
                    });
                    if needs_reopen {
                        gps = Some(GpsReceiver::new(
                            &new_config.gps.device,
                            new_config.gps.baud_rate,
                        ));
                    }
                }
                false => gps = None,
            }

            match new_config.tpms.enabled {
                true => match sensors.as_mut() {
                    Some(s) => s.update_config(&new_config.tpms),
                    None => sensors = Some(BleTpmsReceiver::new(&new_config.tpms)),
                },
                false => sensors = None,
            }
        }

        // 2. Poll hardware GPS; fall back to companion phone GPS via Bluetooth
        let mut has_hardware_fix = false;
        if let Some(gps_ref) = gps.as_mut() {
            if let Some(update) = gps_ref.poll_update() {
                if let Some(speed) = update.speed_kmh {
                    current_speed = Some(speed.max(0.0));
                }
                if let Some(lat) = update.latitude {
                    current_lat = lat;
                }
                if let Some(lon) = update.longitude {
                    current_lon = lon;
                }
                if let Some(hdg) = update.heading {
                    current_heading = hdg;
                }
            }
            has_hardware_fix = gps_ref.is_connected();
            gps_connected = has_hardware_fix;
        }

        if !has_hardware_fix {
            if let Some(phone) = bt_state.as_ref().and_then(|b| b.get_phone_gps()) {
                if let Some(speed) = phone.spd {
                    current_speed = Some(speed.max(0.0));
                }
                if let Some(lat) = phone.lat {
                    current_lat = lat;
                }
                if let Some(lon) = phone.lon {
                    current_lon = lon;
                }
                if let Some(alt) = phone.alt {
                    current_altitude = alt;
                }
                if let Some(hdg) = phone.heading {
                    current_heading = hdg;
                }
                gps_connected = true;
            } else if gps.is_none() {
                gps_connected = false;
            }
        }

        // 3. Update Bluetooth adapter state and connected peer
        let was_active = bt_active;
        bt_active = Path::new("/sys/class/bluetooth/hci0").exists();
        if was_active != bt_active {
            bt_log::log_bt(
                "BT-PROBE",
                &format!(
                    "Bluetooth adapter hci0 state changed: {}",
                    if bt_active {
                        "Active"
                    } else {
                        "Inactive/Missing"
                    }
                ),
            );
        }

        let connected = bt_state.as_ref().and_then(|b| b.connected_peer());
        if bt_device_name != connected {
            match &connected {
                Some(dev) => {
                    bt_log::log_bt("BT-SPP", &format!("Companion phone connected: {}", dev))
                }
                None => {
                    if bt_device_name.is_some() {
                        bt_log::log_bt("BT-SPP", "Companion phone disconnected");
                    }
                }
            }
            bt_device_name = connected;
        }

        // 4. Poll BLE TPMS sensors
        if let Some(ref mut s) = sensors {
            let mut temp_state = VehicleState::default();
            temp_state.tires = current_tires.clone();
            s.poll(&mut temp_state);
            current_tires = temp_state.tires;
        }

        // 5. Expose real-time TPMS telemetry to native Bluetooth server
        if last_tpms_publish.elapsed() >= Duration::from_millis(250) {
            last_tpms_publish = Instant::now();
            if let Some(ref bt) = bt_state {
                bt.set_tires(&current_tires);
            }
        }

        // 6. Poll Bluetooth logs if enabled
        let bt_logs = if log_view_enabled.load(Ordering::Relaxed) {
            bt_log_reader.poll(30).to_vec()
        } else {
            Vec::new()
        };

        // 7. Publish snapshot for main UI thread
        if let Ok(mut lock) = snapshot.lock() {
            lock.current_speed = current_speed;
            lock.gps_latitude = current_lat;
            lock.gps_longitude = current_lon;
            lock.gps_heading = current_heading;
            lock.gps_altitude = current_altitude;
            lock.gps_connected = gps_connected;
            lock.tires = current_tires.clone();
            lock.bt_active = bt_active;
            lock.bt_device_name = bt_device_name.clone();
            if !bt_logs.is_empty() {
                lock.bt_logs = bt_logs;
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}
