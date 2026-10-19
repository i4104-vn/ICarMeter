//! Configuration serialization and updates received from the companion app.

#![allow(dead_code)]

use super::AppConfig;
use std::fs;

pub(super) fn to_toml(config: &AppConfig) -> String {
    format!(
        "# ==============================================================================\n\
         # Car Digital Dashboard - Hardware Configuration for Raspberry Pi OS\n\
         # Synchronized dynamically via ICarMeter Companion\n\
         # ==============================================================================\n\n\
         [gps]\n\
         device = \"{}\"\n\
         baud_rate = {}\n\
         enabled = {}\n\n\
         [tpms]\n\
         enabled = {}\n\
         adapter = \"{}\"\n\
         fl_sensor = \"{}\"\n\
         fr_sensor = \"{}\"\n\
         rl_sensor = \"{}\"\n\
         rr_sensor = \"{}\"\n\
         low_pressure_alert = {:.1}\n\
         high_temp_alert = {:.1}\n\n\
         [display]\n\
         framebuffer_device = \"{}\"\n\
         target_fps = {}\n",
        config.gps.device,
        config.gps.baud_rate,
        config.gps.enabled,
        config.tpms.enabled,
        config.tpms.adapter,
        config.tpms.fl_sensor,
        config.tpms.fr_sensor,
        config.tpms.rl_sensor,
        config.tpms.rr_sensor,
        config.tpms.low_pressure_alert,
        config.tpms.high_temp_alert,
        config.display.framebuffer_device,
        config.display.target_fps
    )
}

pub(super) fn save(config: &AppConfig, path: &str) -> std::io::Result<()> {
    fs::write(path, to_toml(config))
}

pub(super) fn to_json(config: &AppConfig) -> String {
    format!(
        "{{\"gps\":{{\"enabled\":{},\"device\":\"{}\",\"baud_rate\":{}}},\"tpms\":{{\"enabled\":{},\"adapter\":\"{}\",\"fl\":\"{}\",\"fr\":\"{}\",\"rl\":\"{}\",\"rr\":\"{}\",\"low_pressure_alert\":{:.1},\"high_temp_alert\":{:.1}}},\"display\":{{\"framebuffer_device\":\"{}\",\"target_fps\":{}}}}}",
        config.gps.enabled,
        config.gps.device,
        config.gps.baud_rate,
        config.tpms.enabled,
        config.tpms.adapter,
        config.tpms.fl_sensor,
        config.tpms.fr_sensor,
        config.tpms.rl_sensor,
        config.tpms.rr_sensor,
        config.tpms.low_pressure_alert,
        config.tpms.high_temp_alert,
        config.display.framebuffer_device,
        config.display.target_fps
    )
}

pub(super) fn update_from_json(config: &mut AppConfig, json: &str) -> Result<(), String> {
    for (key, target) in [
        ("fl", &mut config.tpms.fl_sensor),
        ("fr", &mut config.tpms.fr_sensor),
        ("rl", &mut config.tpms.rl_sensor),
        ("rr", &mut config.tpms.rr_sensor),
    ] {
        if let Some(value) = extract_string(json, key) {
            *target = value;
        }
    }

    if let Some(alert) =
        extract_float(json, "alert").or_else(|| extract_float(json, "low_pressure_alert"))
    {
        config.tpms.low_pressure_alert = alert;
    }
    if let Some(temp_alert) =
        extract_float(json, "high_temp_alert").or_else(|| extract_float(json, "temp_alert"))
    {
        config.tpms.high_temp_alert = temp_alert;
    }
    if let Some(enabled) = extract_bool(json, "gps_enabled") {
        config.gps.enabled = enabled;
    } else if let Some(mode) = extract_string(json, "gps_mode") {
        config.gps.enabled = mode == "hardware";
    }

    let _ = save(config, "config.toml");
    let _ = save(config, "/opt/icarmeter/config.toml");
    Ok(())
}

fn value_after_key<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{key}\":");
    let start = json.find(&pattern)? + pattern.len();
    Some(json.get(start..)?.trim_start())
}

fn extract_string(json: &str, key: &str) -> Option<String> {
    let value = value_after_key(json, key)?;
    let value = value.strip_prefix('"')?;
    Some(value[..value.find('"')?].to_string())
}

fn extract_float(json: &str, key: &str) -> Option<f32> {
    let value = value_after_key(json, key)?;
    let end = value
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(value.len());
    value[..end].parse().ok()
}

fn extract_bool(json: &str, key: &str) -> Option<bool> {
    let value = value_after_key(json, key)?;
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}
