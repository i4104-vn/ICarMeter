//! Configuration file loading and lightweight TOML parsing.

use super::AppConfig;
use std::fs;

pub(super) fn get_config_mtime() -> Option<std::time::SystemTime> {
    for path in &["config.toml", "/opt/icarmeter/config.toml"] {
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(mtime) = meta.modified() {
                return Some(mtime);
            }
        }
    }
    None
}

pub(super) fn load() -> AppConfig {
    let mut config = AppConfig::default();
    let content = match fs::read_to_string("config.toml")
        .or_else(|_| fs::read_to_string("/opt/icarmeter/config.toml"))
    {
        Ok(content) => content,
        Err(_) => return config,
    };

    let mut section = "";
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        {
            section = name;
            continue;
        }

        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };
        apply_value(&mut config, section, key.trim(), parse_value(raw_value));
    }

    config
}

fn parse_value(raw_value: &str) -> String {
    let value = raw_value.trim();
    if let Some(quoted) = value.strip_prefix('"') {
        return quoted
            .split_once('"')
            .map(|(value, _)| value.to_string())
            .unwrap_or_else(|| quoted.to_string());
    }
    if let Some(quoted) = value.strip_prefix('\'') {
        return quoted
            .split_once('\'')
            .map(|(value, _)| value.to_string())
            .unwrap_or_else(|| quoted.to_string());
    }
    value
        .split_once('#')
        .map(|(value, _)| value.trim())
        .unwrap_or(value)
        .trim_matches(['"', '\''])
        .trim()
        .to_string()
}

fn apply_value(config: &mut AppConfig, section: &str, key: &str, value: String) {
    match section {
        "gps" => match key {
            "device" => config.gps.device = value,
            "baud_rate" => set_if_parses(&value, |baud_rate| config.gps.baud_rate = baud_rate),
            "enabled" => config.gps.enabled = value == "true",
            _ => {}
        },
        "tpms" => match key {
            "enabled" => config.tpms.enabled = value == "true",
            "adapter" => config.tpms.adapter = value,
            "fl_sensor" => config.tpms.fl_sensor = value,
            "fr_sensor" => config.tpms.fr_sensor = value,
            "rl_sensor" => config.tpms.rl_sensor = value,
            "rr_sensor" => config.tpms.rr_sensor = value,
            "low_pressure_alert" => {
                set_if_parses(&value, |alert| config.tpms.low_pressure_alert = alert)
            }
            "high_temp_alert" => {
                set_if_parses(&value, |alert| config.tpms.high_temp_alert = alert)
            }
            _ => {}
        },
        "display" => match key {
            "framebuffer_device" => config.display.framebuffer_device = value,
            "target_fps" => set_if_parses(&value, |fps| config.display.target_fps = fps),
            _ => {}
        },
        _ => {}
    }
}

fn set_if_parses<T>(value: &str, set: impl FnOnce(T))
where
    T: std::str::FromStr,
{
    if let Ok(parsed) = value.parse() {
        set(parsed);
    }
}
