// Hardware and sensor configuration loader for Raspberry Pi OS

use std::fs;

#[derive(Debug, Clone)]
pub struct GpsConfig {
    pub device: String,
    pub baud_rate: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct BleTpmsConfig {
    pub enabled: bool,
    pub adapter: String,
    pub fl_sensor: String,
    pub fr_sensor: String,
    pub rl_sensor: String,
    pub rr_sensor: String,
    pub low_pressure_alert: f32,
}

#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub framebuffer_device: String,
    pub target_fps: u64,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub gps: GpsConfig,
    pub tpms: BleTpmsConfig,
    pub display: DisplayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gps: GpsConfig {
                device: "/dev/serial0".to_string(),
                baud_rate: 9600,
                enabled: true,
            },
            tpms: BleTpmsConfig {
                enabled: true,
                adapter: "hci0".to_string(),
                fl_sensor: "A4:C1:38:00:00:01".to_string(),
                fr_sensor: "A4:C1:38:00:00:02".to_string(),
                rl_sensor: "A4:C1:38:00:00:03".to_string(),
                rr_sensor: "A4:C1:38:00:00:04".to_string(),
                low_pressure_alert: 2.2,
            },
            display: DisplayConfig {
                framebuffer_device: "/dev/fb0".to_string(),
                target_fps: crate::configs::display::TARGET_FPS,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let mut config = Self::default();

        let content = match fs::read_to_string("config.toml") {
            Ok(s) => s,
            Err(_) => return config,
        };

        let mut current_section = "";

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = &trimmed[1..trimmed.len() - 1];
                continue;
            }

            if let Some((key, val_raw)) = trimmed.split_once('=') {
                let key = key.trim();
                let trimmed_val = val_raw.trim();
                let val_str = if trimmed_val.starts_with('"') {
                    if let Some(end_idx) = trimmed_val[1..].find('"') {
                        trimmed_val[1..=end_idx].to_string()
                    } else {
                        trimmed_val.trim_matches('"').to_string()
                    }
                } else if trimmed_val.starts_with('\'') {
                    if let Some(end_idx) = trimmed_val[1..].find('\'') {
                        trimmed_val[1..=end_idx].to_string()
                    } else {
                        trimmed_val.trim_matches('\'').to_string()
                    }
                } else {
                    let before_hash = match trimmed_val.split_once('#') {
                        Some((before, _)) => before.trim(),
                        None => trimmed_val,
                    };
                    before_hash.trim_matches('"').trim_matches('\'').trim().to_string()
                };
                let val = val_str.as_str();

                match current_section {
                    "gps" => match key {
                        "device" => config.gps.device = val.to_string(),
                        "baud_rate" => {
                            if let Ok(b) = val.parse() {
                                config.gps.baud_rate = b;
                            }
                        }
                        "enabled" => config.gps.enabled = val == "true",
                        _ => {}
                    },
                    "tpms" => match key {
                        "enabled" => config.tpms.enabled = val == "true",
                        "adapter" => config.tpms.adapter = val.to_string(),
                        "fl_sensor" => config.tpms.fl_sensor = val.to_string(),
                        "fr_sensor" => config.tpms.fr_sensor = val.to_string(),
                        "rl_sensor" => config.tpms.rl_sensor = val.to_string(),
                        "rr_sensor" => config.tpms.rr_sensor = val.to_string(),
                        "low_pressure_alert" => {
                            if let Ok(p) = val.parse() {
                                config.tpms.low_pressure_alert = p;
                            }
                        }
                        _ => {}
                    },
                    "display" => match key {
                        "framebuffer_device" => config.display.framebuffer_device = val.to_string(),
                        "target_fps" => {
                            if let Ok(fps) = val.parse() {
                                config.display.target_fps = fps;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        config
    }

    /// Serializes configuration back into standardized TOML format
    pub fn to_toml_string(&self) -> String {
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
             low_pressure_alert = {:.1}\n\n\
             [display]\n\
             framebuffer_device = \"{}\"\n\
             target_fps = {}\n",
            self.gps.device,
            self.gps.baud_rate,
            self.gps.enabled,
            self.tpms.enabled,
            self.tpms.adapter,
            self.tpms.fl_sensor,
            self.tpms.fr_sensor,
            self.tpms.rl_sensor,
            self.tpms.rr_sensor,
            self.tpms.low_pressure_alert,
            self.display.framebuffer_device,
            self.display.target_fps
        )
    }

    /// Writes configuration directly to disk on the Raspberry Pi
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        fs::write(path, self.to_toml_string())
    }

    /// Serializes configuration into a compact JSON string for Bluetooth transmission
    pub fn to_json_string(&self) -> String {
        format!(
            "{{\"gps\":{{\"enabled\":{},\"device\":\"{}\",\"baud_rate\":{}}},\"tpms\":{{\"enabled\":{},\"adapter\":\"{}\",\"fl\":\"{}\",\"fr\":\"{}\",\"rl\":\"{}\",\"rr\":\"{}\",\"low_pressure_alert\":{:.1}}},\"display\":{{\"framebuffer_device\":\"{}\",\"target_fps\":{}}}}}",
            self.gps.enabled,
            self.gps.device,
            self.gps.baud_rate,
            self.tpms.enabled,
            self.tpms.adapter,
            self.tpms.fl_sensor,
            self.tpms.fr_sensor,
            self.tpms.rl_sensor,
            self.tpms.rr_sensor,
            self.tpms.low_pressure_alert,
            self.display.framebuffer_device,
            self.display.target_fps
        )
    }

    /// Dynamically updates configuration from a received JSON payload from mobile app
    pub fn update_from_json(&mut self, json_str: &str) -> Result<(), String> {
        // Fast key-value extraction without extra heavy dependencies
        if let Some(fl) = extract_json_str(json_str, "fl") {
            self.tpms.fl_sensor = fl;
        }
        if let Some(fr) = extract_json_str(json_str, "fr") {
            self.tpms.fr_sensor = fr;
        }
        if let Some(rl) = extract_json_str(json_str, "rl") {
            self.tpms.rl_sensor = rl;
        }
        if let Some(rr) = extract_json_str(json_str, "rr") {
            self.tpms.rr_sensor = rr;
        }
        if let Some(alert) = extract_json_float(json_str, "alert")
            .or_else(|| extract_json_float(json_str, "low_pressure_alert"))
        {
            self.tpms.low_pressure_alert = alert;
        }
        if let Some(gps_enabled) = extract_json_bool(json_str, "gps_enabled") {
            self.gps.enabled = gps_enabled;
        } else if let Some(mode) = extract_json_str(json_str, "gps_mode") {
            self.gps.enabled = mode == "hardware";
        }

        // Automatically persist to config.toml on disk
        let _ = self.save("config.toml");
        let _ = self.save("/opt/icarmeter/config.toml");

        Ok(())
    }
}

fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let start_idx = json.find(&pattern)? + pattern.len();
    let rest = json[start_idx..].trim_start();
    if rest.starts_with('"') {
        let content = &rest[1..];
        let end_idx = content.find('"')?;
        Some(content[..end_idx].to_string())
    } else {
        None
    }
}

fn extract_json_float(json: &str, key: &str) -> Option<f32> {
    let pattern = format!("\"{}\":", key);
    let start_idx = json.find(&pattern)? + pattern.len();
    let rest = json[start_idx..].trim_start();
    let end_idx = rest.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(rest.len());
    rest[..end_idx].parse().ok()
}

fn extract_json_bool(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    let start_idx = json.find(&pattern)? + pattern.len();
    let rest = json[start_idx..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}
