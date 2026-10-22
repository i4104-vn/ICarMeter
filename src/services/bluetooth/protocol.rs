//! JSON messaging protocol between ICarMeter and mobile companion app.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::{json, Value};
use crate::configs::AppConfig;
use crate::models::tpms::TirePressures;

pub fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn read_config_json() -> Value {
    let cfg = AppConfig::load();
    json!({
        "tpms": {
            "enabled": cfg.tpms.enabled,
            "adapter": cfg.tpms.adapter,
            "fl": cfg.tpms.fl_sensor,
            "fr": cfg.tpms.fr_sensor,
            "rl": cfg.tpms.rl_sensor,
            "rr": cfg.tpms.rr_sensor,
            "alert": cfg.tpms.low_pressure_alert,
            "high_temp_alert": cfg.tpms.high_temp_alert
        },
        "gps": {
            "enabled": cfg.gps.enabled,
            "device": cfg.gps.device,
            "baud_rate": cfg.gps.baud_rate
        },
        "display": {
            "framebuffer_device": cfg.display.framebuffer_device,
            "target_fps": cfg.display.target_fps
        }
    })
}

pub fn update_config_toml(payload: &Value) -> Result<(), String> {
    let path = AppConfig::path();
    let content = fs::read_to_string(path).unwrap_or_else(|_| crate::configs::config::DEFAULT_CONFIG_CONTENT.to_string());
    
    let mut new_lines = Vec::new();
    let mut in_section = "";

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed.trim_matches(['[', ']']);
            new_lines.push(line.to_string());
            continue;
        }

        if in_section == "tpms" && line.contains('=') {
            if let Some((k, _)) = line.split_once('=') {
                let key = k.trim();
                let updated_line = match key {
                    "fl_sensor" => payload.get("fl").and_then(|v| v.as_str()).map(|v| format!("fl_sensor = \"{}\"", v)),
                    "fr_sensor" => payload.get("fr").and_then(|v| v.as_str()).map(|v| format!("fr_sensor = \"{}\"", v)),
                    "rl_sensor" => payload.get("rl").and_then(|v| v.as_str()).map(|v| format!("rl_sensor = \"{}\"", v)),
                    "rr_sensor" => payload.get("rr").and_then(|v| v.as_str()).map(|v| format!("rr_sensor = \"{}\"", v)),
                    "low_pressure_alert" => payload.get("alert").or_else(|| payload.get("low_pressure_alert"))
                        .and_then(|v| v.as_f64()).map(|v| format!("low_pressure_alert = {:.1}", v)),
                    "high_temp_alert" => payload.get("high_temp_alert").or_else(|| payload.get("temp_alert"))
                        .and_then(|v| v.as_f64()).map(|v| format!("high_temp_alert = {:.1}", v)),
                    _ => None,
                };

                if let Some(l) = updated_line {
                    new_lines.push(l);
                    continue;
                }
            }
        }

        new_lines.push(line.to_string());
    }

    fs::write(path, new_lines.join("\n") + "\n").map_err(|e| e.to_string())
}

pub fn build_welcome(pi_mac: &str, pi_name: &str) -> Value {
    json!({
        "type": "welcome",
        "status": "online",
        "app_connected": true,
        "timestamp": now_timestamp(),
        "config": read_config_json(),
        "pi_mac": pi_mac,
        "pi_name": pi_name
    })
}

pub fn build_pong() -> Value {
    json!({
        "type": "pong",
        "timestamp": now_timestamp()
    })
}

pub fn build_tpms(tires: &TirePressures) -> Value {
    json!({
        "type": "tpms",
        "data": {
            "fl": tires.fl,
            "fr": tires.fr,
            "rl": tires.rl,
            "rr": tires.rr,
            "fl_temp": tires.fl_temp,
            "fr_temp": tires.fr_temp,
            "rl_temp": tires.rl_temp,
            "rr_temp": tires.rr_temp,
            "alert": tires.low_pressure_alert,
            "high_temp_alert": tires.high_temp_alert
        }
    })
}

pub fn build_heartbeat(clients: usize) -> Value {
    json!({
        "type": "heartbeat",
        "status": "online",
        "clients": clients,
        "timestamp": now_timestamp()
    })
}

pub fn build_status(clients: usize) -> Value {
    json!({
        "type": "status",
        "data": {
            "status": "online",
            "app_connected": clients > 0,
            "clients": clients,
            "timestamp": now_timestamp()
        }
    })
}

pub fn build_pi_matched(pi_mac: &str, pi_name: &str, channel: u8) -> Value {
    json!({
        "type": "pi_matched",
        "pi_mac": pi_mac,
        "pi_name": pi_name,
        "rfcomm_channel": channel,
        "timestamp": now_timestamp()
    })
}

pub fn build_bt_status(pi_mac: &str, pi_name: &str, paired_devices: Vec<Value>) -> Value {
    json!({
        "type": "bt_status",
        "data": {
            "controller_powered": true,
            "discoverable": true,
            "pairable": true,
            "pi_mac": pi_mac,
            "pi_name": pi_name,
            "paired_devices": paired_devices
        },
        "pi_mac": pi_mac,
        "pi_name": pi_name
    })
}
