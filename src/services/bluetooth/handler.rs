//! Dedicated connection handler processing commands from connected mobile companion app.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use std::time::Duration;

use super::adapter;
use super::protocol;
use super::server::Broadcaster;
use super::socket::RfcommStream;
use crate::models::bluetooth::state::BluetoothSharedState;
use crate::models::gps::phone::PhoneGpsData;
use crate::services::telemetry::bt_log::{log_bt, read_recent_bt_logs};

pub fn handle_client(
    mut stream: RfcommStream,
    shared_state: BluetoothSharedState,
    broadcaster: Arc<Broadcaster>,
) {
    let peer_addr = stream.peer_addr().to_string();
    log_bt("BT-RFCOMM", &format!("Client connected from {}", peer_addr));
    let _ = stream.set_read_timeout(Some(Duration::from_secs(60)));

    shared_state.set_client_connected(&peer_addr);

    let pi_mac = adapter::get_local_mac(adapter::DEFAULT_ADAPTER);
    let welcome = protocol::build_welcome(&pi_mac, "ICarMeter");
    let _ = send_json_line(&mut stream, &welcome);

    let reader_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            log_bt(
                "BT-RFCOMM",
                &format!("Failed to clone stream for reader: {}", e),
            );
            cleanup_client(&shared_state);
            return;
        }
    };

    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF: client disconnected
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Ok(msg) = serde_json::from_str::<Value>(trimmed) {
                    process_message(&mut stream, &msg, &pi_mac, &shared_state, &broadcaster);
                }
            }
            Err(e) => {
                // If it's a timeout, send a ping to keep connection alive
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut
                {
                    if send_json_line(&mut stream, &json!({"type": "ping"})).is_err() {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
    }

    log_bt(
        "BT-RFCOMM",
        &format!("Client disconnected from {}", peer_addr),
    );
    cleanup_client(&shared_state);
}

fn cleanup_client(shared_state: &BluetoothSharedState) {
    shared_state.set_client_disconnected();
}

fn send_json_line(stream: &mut RfcommStream, val: &Value) -> std::io::Result<()> {
    let mut data = serde_json::to_vec(val)?;
    data.push(b'\n');
    stream.write_all(&data)?;
    stream.flush()
}

fn process_message(
    stream: &mut RfcommStream,
    msg: &Value,
    pi_mac: &str,
    shared_state: &BluetoothSharedState,
    broadcaster: &Arc<Broadcaster>,
) {
    let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match msg_type {
        "ping" => {
            let _ = send_json_line(stream, &protocol::build_pong());
        }
        "get_config" => {
            let resp = json!({
                "type": "config",
                "data": protocol::read_config_json()
            });
            let _ = send_json_line(stream, &resp);
        }
        "set_config" => {
            if let Some(payload) = msg.get("payload") {
                if let Err(e) = protocol::update_config_toml(payload) {
                    let _ = send_json_line(stream, &json!({"type": "error", "message": e}));
                } else {
                    let updated = json!({
                        "type": "config",
                        "data": protocol::read_config_json(),
                        "message": "Saved",
                        "status": "ok"
                    });
                    broadcaster.broadcast(&updated);
                }
            }
        }
        "get_tpms" => {
            let _ = send_json_line(stream, &protocol::build_tpms(&shared_state.get_tires()));
        }
        "telemetry" | "gps" => {
            let spd = msg.get("spd").and_then(|v| v.as_f64()).map(|v| v as f32);
            let lat = msg.get("lat").and_then(|v| v.as_f64());
            let lon = msg.get("lon").and_then(|v| v.as_f64());
            let alt = msg.get("alt").and_then(|v| v.as_f64()).map(|v| v as f32);
            let heading = msg
                .get("heading")
                .or_else(|| msg.get("bearing"))
                .and_then(|v| v.as_f64())
                .map(|v| v as f32);

            let phone_gps = PhoneGpsData::new(spd, lat, lon, alt, heading);
            shared_state.set_phone_gps(phone_gps);
        }
        "get_status" => {
            let _ = send_json_line(
                stream,
                &protocol::build_status(shared_state.active_clients()),
            );
        }
        "match_pi" | "get_pi_info" => {
            adapter::ensure_adapter_ready(adapter::DEFAULT_ADAPTER);
            let _ = send_json_line(stream, &protocol::build_pi_matched(pi_mac, "ICarMeter", 1));
        }
        "get_bt_status" => {
            let paired = adapter::get_paired_devices();
            let _ = send_json_line(
                stream,
                &protocol::build_bt_status(pi_mac, "ICarMeter", paired),
            );
        }
        "get_bt_logs" => {
            let logs = read_recent_bt_logs(100);
            let _ = send_json_line(stream, &json!({"type": "bt_logs", "logs": logs}));
        }
        "clear_bt_logs" => {
            let _ = send_json_line(stream, &json!({"type": "bt_logs", "logs": []}));
        }
        "bt_start_discovery" => {
            adapter::start_discovery();
            let _ = send_json_line(stream, &json!({"type": "bt_discovery_started"}));
        }
        "bt_stop_discovery" => {
            adapter::stop_discovery();
            let _ = send_json_line(stream, &json!({"type": "bt_discovery_stopped"}));
        }
        "bt_pair" => {
            if let Some(target_mac) = msg.get("mac").and_then(|v| v.as_str()) {
                let mac = target_mac.to_string();
                let b_cast = broadcaster.clone();
                std::thread::spawn(move || {
                    let (ok, detail) = adapter::pair_device(&mac);
                    b_cast.broadcast(&json!({
                        "type": "bt_pair_result",
                        "mac": mac,
                        "success": ok,
                        "message": detail
                    }));
                });
            }
        }
        "bt_unpair" => {
            if let Some(target_mac) = msg.get("mac").and_then(|v| v.as_str()) {
                let ok = adapter::unpair_device(target_mac);
                let _ = send_json_line(
                    stream,
                    &json!({
                        "type": "bt_unpair_result",
                        "mac": target_mac,
                        "success": ok
                    }),
                );
            }
        }
        _ => {
            log_bt(
                "BT-RFCOMM",
                &format!("Unhandled command type: {}", msg_type),
            );
        }
    }
}
