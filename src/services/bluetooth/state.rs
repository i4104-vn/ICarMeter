//! Thread-safe shared state between Bluetooth service and main telemetry loop.

use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::models::tpms::TirePressures;

#[derive(Debug, Clone, Default)]
pub struct PhoneGpsData {
    pub spd: Option<f32>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub alt: Option<f32>,
    pub heading: Option<f32>,
    pub last_updated: Option<Instant>,
}

#[derive(Debug, Clone, Default)]
struct InnerState {
    tires: TirePressures,
    phone_gps: Option<PhoneGpsData>,
    connected_peer: Option<String>,
    active_clients: usize,
}

#[derive(Debug, Clone)]
pub struct BluetoothSharedState {
    inner: Arc<Mutex<InnerState>>,
}

impl Default for BluetoothSharedState {
    fn default() -> Self {
        Self::new()
    }
}

impl BluetoothSharedState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerState::default())),
        }
    }

    pub fn set_tires(&self, tires: &TirePressures) {
        if let Ok(mut lock) = self.inner.lock() {
            lock.tires = *tires;
        }
    }

    pub fn get_tires(&self) -> TirePressures {
        self.inner
            .lock()
            .map(|l| l.tires)
            .unwrap_or_default()
    }

    pub fn set_phone_gps(&self, gps: PhoneGpsData) {
        if let Ok(mut lock) = self.inner.lock() {
            lock.phone_gps = Some(gps);
        }
    }

    pub fn take_phone_gps(&self) -> Option<PhoneGpsData> {
        self.inner.lock().ok().and_then(|mut l| l.phone_gps.take())
    }

    pub fn set_client_connected(&self, peer_name: &str) {
        if let Ok(mut lock) = self.inner.lock() {
            lock.active_clients += 1;
            lock.connected_peer = Some(peer_name.to_string());
        }
    }

    pub fn set_client_disconnected(&self) {
        if let Ok(mut lock) = self.inner.lock() {
            lock.active_clients = lock.active_clients.saturating_sub(1);
            if lock.active_clients == 0 {
                lock.connected_peer = None;
            }
        }
    }

    pub fn active_clients(&self) -> usize {
        self.inner.lock().map(|l| l.active_clients).unwrap_or(0)
    }

    pub fn connected_peer(&self) -> Option<String> {
        self.inner.lock().ok().and_then(|l| l.connected_peer.clone())
    }
}
