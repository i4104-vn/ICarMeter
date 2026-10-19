// Wireless Bluetooth Low Energy (BLE) TPMS telemetry module

pub mod decoder;
pub mod hci_socket;
pub mod scanner;

use crate::configs::app::BleTpmsConfig;
use crate::models::VehicleState;
use scanner::{SharedTirePressures, run_ble_scanner_loop};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

pub struct BleTpmsReceiver {
    pressures: Arc<Mutex<SharedTirePressures>>,
    expression: Arc<Mutex<Option<String>>>,
    config: Arc<RwLock<BleTpmsConfig>>,
}

impl BleTpmsReceiver {
    pub fn new(config: &BleTpmsConfig) -> Self {
        let pressures = Arc::new(Mutex::new(SharedTirePressures::default()));
        let expression = Arc::new(Mutex::new(None));
        let shared_config = Arc::new(RwLock::new(config.clone()));

        let thread_pressures = Arc::clone(&pressures);
        let thread_expression = Arc::clone(&expression);
        let thread_config = Arc::clone(&shared_config);

        thread::Builder::new()
            .name("bt-tpms-scanner".to_string())
            .spawn(move || {
                run_ble_scanner_loop(thread_config, thread_pressures, thread_expression);
            })
            .expect("Failed to spawn Bluetooth TPMS scanner thread");

        Self {
            pressures,
            expression,
            config: shared_config,
        }
    }

    pub fn update_config(&mut self, new_config: &BleTpmsConfig) {
        if let Ok(mut lock) = self.config.write() {
            *lock = new_config.clone();
        }
    }

    /// Non-blocking update called every frame on the render loop (60 FPS)
    pub fn poll(&mut self, state: &mut VehicleState) {
        if let Ok(guard) = self.pressures.try_lock() {
            if guard.fl > 0.0 {
                state.tires.fl = guard.fl;
            }
            if guard.fr > 0.0 {
                state.tires.fr = guard.fr;
            }
            if guard.rl > 0.0 {
                state.tires.rl = guard.rl;
            }
            if guard.rr > 0.0 {
                state.tires.rr = guard.rr;
            }
            if guard.fl_temp != 0.0 {
                state.tires.fl_temp = guard.fl_temp;
            }
            if guard.fr_temp != 0.0 {
                state.tires.fr_temp = guard.fr_temp;
            }
            if guard.rl_temp != 0.0 {
                state.tires.rl_temp = guard.rl_temp;
            }
            if guard.rr_temp != 0.0 {
                state.tires.rr_temp = guard.rr_temp;
            }
        }

        if let Ok(mut expr_guard) = self.expression.try_lock() {
            if let Some(ref name) = *expr_guard {
                state.set_expression_name(name);
            }
            *expr_guard = None;
        }
    }
}
