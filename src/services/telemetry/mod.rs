// Telemetry service coordinator running on dedicated background worker thread

pub mod gps;

use crate::configs::app::AppConfig;
use crate::models::tpms::TirePressures;
use crate::models::vehicle::VehicleState;
use crate::services::is_app_running;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub use gps::GpsReceiver;

#[derive(Debug, Default)]
pub struct TelemetrySnapshot {
    pub current_speed: Option<f32>,
    pub gps_latitude: f64,
    pub gps_longitude: f64,
    pub gps_heading: f32,
    pub gps_altitude: f32,
    pub gps_connected: bool,
    pub tires: TirePressures,
}

pub struct TelemetryService {
    snapshot: Arc<Mutex<TelemetrySnapshot>>,
    config_tx: Sender<AppConfig>,
}

impl TelemetryService {
    pub fn new(config: &AppConfig) -> Self {
        let snapshot = Arc::new(Mutex::new(TelemetrySnapshot::default()));
        let (config_tx, config_rx) = mpsc::channel::<AppConfig>();

        let thread_snapshot = Arc::clone(&snapshot);
        let initial_config = config.clone();

        thread::Builder::new()
            .name("telemetry-worker".to_string())
            .spawn(move || {
                run_telemetry_loop(
                    initial_config,
                    config_rx,
                    thread_snapshot,
                );
            })
            .expect("Failed to spawn telemetry-worker thread");

        Self {
            snapshot,
            config_tx,
        }
    }

    pub fn reload_config(&mut self, config: &AppConfig) {
        let _ = self.config_tx.send(config.clone());
    }

    /// Fast, non-blocking state sync called from main UI render thread.
    /// Performs zero I/O, zero syscalls, zero blocking locks.
    pub fn update(&mut self, state: &mut VehicleState) {
        if let Ok(guard) = self.snapshot.lock() {
            if let Some(speed) = guard.current_speed {
                state.current_speed = speed;
            }
            state.gps_latitude = guard.gps_latitude;
            state.gps_longitude = guard.gps_longitude;
            state.gps_heading = guard.gps_heading;
            state.gps_altitude = guard.gps_altitude;
            state.gps_connected = guard.gps_connected;
            state.tires = guard.tires;
        }
    }
}

/// Independent telemetry loop handling hardware GPS serial I/O off the main thread.
fn run_telemetry_loop(
    config: AppConfig,
    config_rx: Receiver<AppConfig>,
    snapshot: Arc<Mutex<TelemetrySnapshot>>,
) {
    let mut gps = if config.gps.enabled {
        Some(GpsReceiver::new(&config.gps.device, config.gps.baud_rate))
    } else {
        None
    };

    let current_tires = TirePressures::default();
    let mut current_speed: Option<f32> = None;
    let mut current_lat = 21.0285;
    let mut current_lon = 105.8542;
    let mut current_heading = 0.0;
    let current_altitude = 18.0;
    let mut gps_connected;

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
        }

        // 2. Poll hardware GPS serial receiver
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
            gps_connected = gps_ref.is_connected();
        } else {
            gps_connected = false;
        }

        // 3. Publish snapshot for main UI thread
        if let Ok(mut lock) = snapshot.lock() {
            lock.current_speed = current_speed;
            lock.gps_latitude = current_lat;
            lock.gps_longitude = current_lon;
            lock.gps_heading = current_heading;
            lock.gps_altitude = current_altitude;
            lock.gps_connected = gps_connected;
            lock.tires = current_tires;
        }

        thread::sleep(Duration::from_millis(50));
    }
}
