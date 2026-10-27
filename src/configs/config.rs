//! Single configuration path manager, auto-generator, and TOML parser.
//!
//! Enforces a single, non-fallback configuration path (~/.config/icarmeter/config.toml)
//! across the entire application. If the file does not exist, it is automatically
//! created with default settings and read/write permissions for all users (0o666).

use super::app::AppConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::SystemTime;

pub const DEFAULT_CONFIG_CONTENT: &str = r#"# ==============================================================================
# Car Digital Dashboard - Hardware Configuration for Raspberry Pi OS
# ==============================================================================

[gps]
# Cổng serial GPS UART phần cứng (nối qua chân GPIO Tx/Rx /dev/serial0)
device = "/dev/serial0"
baud_rate = 9600
enabled = true

[tpms]
# Kích hoạt hệ thống cảm biến áp suất lốp
enabled = false

# Ngưỡng cảnh báo áp suất lốp thấp (bar)
low_pressure_alert = 2.2

# Ngưỡng cảnh báo nhiệt độ lốp cao (°C)
high_temp_alert = 70.0

[display]
target_fps = 60
"#;

static CONFIG_PATH_CACHE: OnceLock<PathBuf> = OnceLock::new();

/// Resolves the single, unified configuration file path across the application.
/// Checks user environment (~/.config/icarmeter/config.toml) without fallback.
fn resolve_config_path() -> PathBuf {
    if let Ok(sudo_user) = std::env::var("SUDO_USER")
        && !sudo_user.is_empty()
    {
        return PathBuf::from(format!("/home/{sudo_user}/.config/icarmeter/config.toml"));
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        return PathBuf::from(xdg).join("icarmeter/config.toml");
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
        && home != "/root"
    {
        return PathBuf::from(home).join(".config/icarmeter/config.toml");
    }
    if Path::new("/home/pi/.config").is_dir() || Path::new("/home/pi").is_dir() {
        return PathBuf::from("/home/pi/.config/icarmeter/config.toml");
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        return PathBuf::from(home).join(".config/icarmeter/config.toml");
    }
    PathBuf::from("/etc/xdg/icarmeter/config.toml")
}

/// Returns a reference to the single static configuration path.
pub fn get_config_path() -> &'static Path {
    CONFIG_PATH_CACHE.get_or_init(resolve_config_path).as_path()
}

/// Ensures the configuration file exists at the single target path.
/// If absent, automatically creates parent directories and generates the default config.toml
/// with read/write permissions (0o666 on Unix) so any user or service can read/write it.
pub fn ensure_config_exists() -> &'static Path {
    let path = get_config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "[Config] Warning: Failed to create config directory {:?}: {}",
                    parent, e
                );
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o777));
            }
        }

        match fs::write(path, DEFAULT_CONFIG_CONTENT) {
            Ok(_) => {
                println!(
                    "[Config] Generated default configuration file at: {}",
                    path.display()
                );
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o666));
                }
            }
            Err(e) => {
                eprintln!(
                    "[Config] Error: Failed to write default config to {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }
    path
}

/// Gets last modified timestamp of the single configuration file.
pub fn get_config_mtime() -> Option<SystemTime> {
    let path = get_config_path();
    if let Ok(meta) = fs::metadata(path)
        && let Ok(mtime) = meta.modified()
    {
        return Some(mtime);
    }
    None
}

/// Loads application configuration from the single configuration file,
/// automatically generating it if it does not exist yet.
pub fn load() -> AppConfig {
    let path = ensure_config_exists();
    let mut config = AppConfig::default();

    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "[Config] Failed to read config file {}: {}",
                path.display(),
                e
            );
            return config;
        }
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
            "high_temp_alert" => set_if_parses(&value, |alert| config.tpms.high_temp_alert = alert),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_template_validity() {
        assert!(DEFAULT_CONFIG_CONTENT.contains("[gps]"));
        assert!(DEFAULT_CONFIG_CONTENT.contains("[tpms]"));
        assert!(DEFAULT_CONFIG_CONTENT.contains("[display]"));
        assert!(DEFAULT_CONFIG_CONTENT.contains("low_pressure_alert = 2.2"));
    }

    #[test]
    fn test_parse_value() {
        assert_eq!(parse_value("\"A4:C1:38:00:00:01\""), "A4:C1:38:00:00:01");
        assert_eq!(parse_value("'hci0'"), "hci0");
        assert_eq!(parse_value("60 # Target framerate"), "60");
        assert_eq!(parse_value("true"), "true");
        assert_eq!(parse_value("2.2"), "2.2");
    }

    #[test]
    fn test_config_path_format() {
        let path = get_config_path();
        assert!(path.ends_with("icarmeter/config.toml"));
    }

    #[test]
    fn test_ensure_and_load_config() {
        let path = ensure_config_exists();
        assert!(path.exists());
        let config = load();
        assert_eq!(config.gps.baud_rate, 9600);
        assert_eq!(config.tpms.low_pressure_alert, 2.2);
        assert_eq!(config.display.target_fps, 60);
    }
}
