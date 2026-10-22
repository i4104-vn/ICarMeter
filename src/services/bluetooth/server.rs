//! RFCOMM server listener, connection registry, and telemetry broadcaster.

use std::io::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde_json::Value;

use super::adapter;
use super::handler;
use super::protocol;
use super::socket::{RfcommListener, RfcommStream, RFCOMM_CHANNEL_DEFAULT};
use super::state::BluetoothSharedState;
use crate::services::telemetry::bt_log::log_bt;

pub struct Broadcaster {
    clients: Mutex<Vec<RfcommStream>>,
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(Vec::new()),
        }
    }

    pub fn register(&self, stream: RfcommStream) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.push(stream);
        }
    }

    pub fn broadcast(&self, val: &Value) {
        let Ok(mut data) = serde_json::to_vec(val) else {
            return;
        };
        data.push(b'\n');

        let mut dead_indices = Vec::new();
        if let Ok(mut clients) = self.clients.lock() {
            for (idx, client) in clients.iter_mut().enumerate() {
                if client.write_all(&data).is_err() || client.flush().is_err() {
                    dead_indices.push(idx);
                }
            }
            for idx in dead_indices.into_iter().rev() {
                clients.swap_remove(idx);
            }
        }
    }
}

pub struct RfcommServer {
    shared_state: BluetoothSharedState,
    broadcaster: Arc<Broadcaster>,
}

impl RfcommServer {
    pub fn start(shared_state: BluetoothSharedState) -> Self {
        let broadcaster = Arc::new(Broadcaster::new());

        let listener_state = shared_state.clone();
        let listener_broadcaster = broadcaster.clone();
        thread::Builder::new()
            .name("bt-rfcomm-listener".into())
            .spawn(move || {
                run_listener_loop(listener_state, listener_broadcaster);
            })
            .expect("Failed to spawn bt-rfcomm-listener thread");

        let hb_state = shared_state.clone();
        let hb_broadcaster = broadcaster.clone();
        thread::Builder::new()
            .name("bt-heartbeat".into())
            .spawn(move || {
                run_heartbeat_loop(hb_state, hb_broadcaster);
            })
            .expect("Failed to spawn bt-heartbeat thread");

        Self {
            shared_state,
            broadcaster,
        }
    }

    pub fn shared_state(&self) -> &BluetoothSharedState {
        &self.shared_state
    }

    pub fn broadcaster(&self) -> &Arc<Broadcaster> {
        &self.broadcaster
    }
}

fn register_sdp_spp(channel: u8) {
    let _ = Command::new("sdptool").args(["add", &format!("--channel={}", channel), "SP"]).output();
}

fn run_listener_loop(shared_state: BluetoothSharedState, broadcaster: Arc<Broadcaster>) {
    loop {
        adapter::ensure_adapter_ready(adapter::DEFAULT_ADAPTER);
        register_sdp_spp(RFCOMM_CHANNEL_DEFAULT);

        match RfcommListener::bind(RFCOMM_CHANNEL_DEFAULT) {
            Ok(listener) => {
                log_bt("BT-RFCOMM", &format!("RFCOMM server permanently listening on channel {}...", listener.channel()));
                loop {
                    match listener.accept() {
                        Ok(stream) => {
                            if let Ok(cloned_stream) = stream.try_clone() {
                                broadcaster.register(cloned_stream);
                            }
                            let client_state = shared_state.clone();
                            let client_broadcaster = broadcaster.clone();
                            thread::spawn(move || {
                                handler::handle_client(stream, client_state, client_broadcaster);
                            });
                        }
                        Err(e) => {
                            log_bt("BT-RFCOMM", &format!("Accept error: {}. Restarting listener...", e));
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                log_bt("BT-RFCOMM", &format!("Bind error: {}. Retrying in 3s...", e));
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}

fn run_heartbeat_loop(shared_state: BluetoothSharedState, broadcaster: Arc<Broadcaster>) {
    let mut tick: u64 = 0;
    loop {
        thread::sleep(Duration::from_secs(2));
        tick = tick.wrapping_add(1);

        let clients = shared_state.active_clients();
        if clients > 0 {
            // 1. Broadcast real-time TPMS telemetry
            let tpms_msg = protocol::build_tpms(&shared_state.get_tires());
            broadcaster.broadcast(&tpms_msg);

            // 2. Broadcast heartbeat every 4 seconds
            if tick % 2 == 0 {
                let hb_msg = protocol::build_heartbeat(clients);
                broadcaster.broadcast(&hb_msg);
            }
        }

        // Periodically verify adapter remains discoverable & pairable
        if tick % 15 == 0 {
            adapter::ensure_adapter_ready(adapter::DEFAULT_ADAPTER);
        }
    }
}
