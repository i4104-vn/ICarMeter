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

            if let Some((key, val)) = trimmed.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');

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
}
