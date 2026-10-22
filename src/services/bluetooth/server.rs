//! BlueZ profile and pairing agent, connection registry, and telemetry broadcaster.

use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zbus::blocking::{Connection, Proxy};
use zbus::interface;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value as ZValue};

use super::adapter;
use super::handler;
use super::protocol;
use super::socket::RfcommStream;
use crate::models::bluetooth::address::RFCOMM_CHANNEL_DEFAULT;
use crate::models::bluetooth::state::BluetoothSharedState;
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

pub struct RfcommServer;

const PROFILE_MANAGER_INTERFACE: &str = "org.bluez.ProfileManager1";
const PROFILE_PATH: &str = "/org/icarmeter/bluetooth/spp";
const SPP_UUID: &str = "00001101-0000-1000-8000-00805F9B34FB";
const AGENT_MANAGER_INTERFACE: &str = "org.bluez.AgentManager1";
const AGENT_PATH: &str = "/org/icarmeter/bluetooth/agent";

impl RfcommServer {
    pub fn start(shared_state: BluetoothSharedState) -> Self {
        let broadcaster = Arc::new(Broadcaster::new());

        let listener_state = shared_state.clone();
        let listener_broadcaster = broadcaster.clone();
        thread::Builder::new()
            .name("bt-rfcomm-listener".into())
            .spawn(move || {
                run_profile_loop(listener_state, listener_broadcaster);
            })
            .expect("Failed to spawn bt-rfcomm-listener thread");

        let hb_state = shared_state;
        let hb_broadcaster = broadcaster;
        thread::Builder::new()
            .name("bt-heartbeat".into())
            .spawn(move || {
                run_heartbeat_loop(hb_state, hb_broadcaster);
            })
            .expect("Failed to spawn bt-heartbeat thread");

        Self
    }
}

struct SppProfile {
    shared_state: BluetoothSharedState,
    broadcaster: Arc<Broadcaster>,
}

#[interface(name = "org.bluez.Profile1")]
impl SppProfile {
    fn release(&self) {}

    fn new_connection(
        &self,
        device: OwnedObjectPath,
        fd: zbus::zvariant::OwnedFd,
        _fd_properties: HashMap<String, OwnedValue>,
    ) {
        let peer_addr = device
            .as_str()
            .rsplit("/dev_")
            .next()
            .unwrap_or(device.as_str())
            .replace('_', ":")
            .to_uppercase();
        let stream = RfcommStream::from_owned_fd(fd.into(), peer_addr);

        if let Ok(broadcast_stream) = stream.try_clone() {
            self.broadcaster.register(broadcast_stream);
        }

        let shared_state = self.shared_state.clone();
        let broadcaster = self.broadcaster.clone();
        thread::spawn(move || {
            handler::handle_client(stream, shared_state, broadcaster);
        });
    }
}

struct PairingAgent;

#[interface(name = "org.bluez.Agent1")]
impl PairingAgent {
    fn release(&self) {}

    fn request_pin_code(&self, device: OwnedObjectPath) -> zbus::fdo::Result<String> {
        log_bt("BT-AGENT", &format!("PIN requested by {device}, auto-replying 0000"));
        Ok("0000".to_string())
    }

    fn display_pin_code(&self, device: OwnedObjectPath, pin_code: String) {
        log_bt("BT-AGENT", &format!("PIN for {device}: {pin_code}"));
    }

    fn request_passkey(&self, device: OwnedObjectPath) -> zbus::fdo::Result<u32> {
        log_bt("BT-AGENT", &format!("Passkey requested by {device}, auto-replying 0"));
        Ok(0)
    }

    fn display_passkey(&self, device: OwnedObjectPath, passkey: u32, _entered: u16) {
        log_bt("BT-AGENT", &format!("Passkey for {device}: {passkey:06}"));
    }

    fn request_confirmation(&self, device: OwnedObjectPath, passkey: u32) {
        log_bt("BT-AGENT", &format!("Auto-confirmed pairing passkey {passkey:06} for {device}"));
    }

    fn request_authorization(&self, device: OwnedObjectPath) {
        log_bt("BT-AGENT", &format!("Authorized connection from {device}"));
    }

    fn authorize_service(&self, device: OwnedObjectPath, uuid: String) {
        log_bt("BT-AGENT", &format!("Authorized service {uuid} for {device}"));
    }

    fn cancel(&self) {
        log_bt("BT-AGENT", "Pairing cancelled");
    }
}

fn run_profile_loop(shared_state: BluetoothSharedState, broadcaster: Arc<Broadcaster>) {
    loop {
        adapter::ensure_adapter_ready(adapter::DEFAULT_ADAPTER);
        match register_spp_profile(shared_state.clone(), broadcaster.clone()) {
            Ok((connection, bluez_owner)) => {
                log_bt("BT-RFCOMM", "SPP profile registered with BlueZ over D-Bus");
                loop {
                    thread::sleep(Duration::from_secs(10));
                    if !matches!(
                        current_bluez_owner(&connection),
                        Ok(owner) if owner == bluez_owner
                    ) {
                        log_bt(
                            "BT-RFCOMM",
                            "BlueZ owner changed; re-registering SPP profile",
                        );
                        break;
                    }
                }
            }
            Err(error) => {
                log_bt(
                    "BT-RFCOMM",
                    &format!("SPP profile registration failed: {error}"),
                );
                thread::sleep(Duration::from_secs(3));
            }
        }
    }
}

fn register_spp_profile(
    shared_state: BluetoothSharedState,
    broadcaster: Arc<Broadcaster>,
) -> Result<(Connection, String), Box<dyn std::error::Error>> {
    let connection = Connection::system()?;
    connection.object_server().at(
        PROFILE_PATH,
        SppProfile {
            shared_state,
            broadcaster,
        },
    )?;
    connection.object_server().at(AGENT_PATH, PairingAgent)?;

    if let Err(error) = register_pairing_agent(&connection) {
        log_bt(
            "BT-AGENT",
            &format!("Could not register default pairing agent: {error}"),
        );
    }

    let mut options = HashMap::new();
    options.insert(
        "Name".to_string(),
        OwnedValue::try_from(ZValue::from("ICarMeter SPP"))?,
    );
    options.insert(
        "Role".to_string(),
        OwnedValue::try_from(ZValue::from("server"))?,
    );
    options.insert(
        "Channel".to_string(),
        OwnedValue::from(RFCOMM_CHANNEL_DEFAULT as u16),
    );
    options.insert(
        "AutoConnect".to_string(),
        OwnedValue::from(true),
    );
    options.insert(
        "RequireAuthentication".to_string(),
        OwnedValue::from(false),
    );
    options.insert(
        "RequireAuthorization".to_string(),
        OwnedValue::from(false),
    );

    let manager = Proxy::new(
        &connection,
        "org.bluez",
        "/org/bluez",
        PROFILE_MANAGER_INTERFACE,
    )?;
    let profile_path = ObjectPath::try_from(PROFILE_PATH)?;
    manager.call_method("RegisterProfile", &(profile_path, SPP_UUID, options))?;

    let bluez_owner = current_bluez_owner(&connection)?;
    Ok((connection, bluez_owner))
}

fn register_pairing_agent(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let manager = Proxy::new(
        connection,
        "org.bluez",
        "/org/bluez",
        AGENT_MANAGER_INTERFACE,
    )?;
    let agent_path = ObjectPath::try_from(AGENT_PATH)?;
    manager.call_method("RegisterAgent", &(agent_path.clone(), "NoInputNoOutput"))?;
    manager.call_method("RequestDefaultAgent", &(agent_path,))?;
    log_bt("BT-AGENT", "Auto-pairing agent registered with BlueZ over D-Bus");
    Ok(())
}

fn current_bluez_owner(connection: &Connection) -> Result<String, Box<dyn std::error::Error>> {
    let bus = Proxy::new(
        connection,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )?;
    let reply = bus.call_method("GetNameOwner", &("org.bluez",))?;
    Ok(reply.body().deserialize()?)
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
            if tick.is_multiple_of(2) {
                let hb_msg = protocol::build_heartbeat(clients);
                broadcaster.broadcast(&hb_msg);
            }
        }

        // Periodically verify adapter remains discoverable & pairable over D-Bus
        if tick.is_multiple_of(5) {
            adapter::maintain_discoverability(adapter::DEFAULT_ADAPTER);
        }
    }
}
