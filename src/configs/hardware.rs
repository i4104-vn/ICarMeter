// Hardware and sensor configuration loader for Raspberry Pi OS

use std::fs;

#[derive(Debug, Clone)]
pub struct GpsConfig {
    pub device: String,
    pub baud_rate: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SensorConfig {
    pub device: String,
    pub baud_rate: u32,
    pub enabled: bool,
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
    pub sensors: SensorConfig,
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
            sensors: SensorConfig {
                device: "/dev/rfcomm0".to_string(),
                baud_rate: 9600,
                enabled: true,
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
                    "sensors" => match key {
                        "device" => config.sensors.device = val.to_string(),
                        "baud_rate" => {
                            if let Ok(b) = val.parse() {
                                config.sensors.baud_rate = b;
                            }
                        }
                        "enabled" => config.sensors.enabled = val == "true",
                        "low_pressure_alert" => {
                            if let Ok(p) = val.parse() {
                                config.sensors.low_pressure_alert = p;
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
