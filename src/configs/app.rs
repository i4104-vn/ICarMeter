//! Application configuration model and public configuration API.

const DEFAULT_TARGET_FPS: u64 = 60;

#[derive(Debug, Clone)]
pub struct GpsConfig {
    pub device: String,
    pub baud_rate: u32,
    pub enabled: bool,
    pub bluetooth: bool,
}

#[derive(Debug, Clone)]
pub struct TpmsConfig {
    pub enabled: bool,
    pub adapter: String,
    pub fl_sensor: String,
    pub fr_sensor: String,
    pub rl_sensor: String,
    pub rr_sensor: String,
    pub low_pressure_alert: f32,
    pub high_temp_alert: f32,
}

#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub target_fps: u64,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub gps: GpsConfig,
    pub tpms: TpmsConfig,
    pub display: DisplayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gps: GpsConfig {
                device: "/dev/serial0".to_string(),
                baud_rate: 9600,
                enabled: true,
                bluetooth: true,
            },
            tpms: TpmsConfig {
                enabled: false,
                adapter: "hci0".to_string(),
                fl_sensor: String::new(),
                fr_sensor: String::new(),
                rl_sensor: String::new(),
                rr_sensor: String::new(),
                low_pressure_alert: 2.2,
                high_temp_alert: 70.0,
            },
            display: DisplayConfig {
                target_fps: DEFAULT_TARGET_FPS,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        crate::configs::config::load()
    }

    pub fn last_modified() -> Option<std::time::SystemTime> {
        crate::configs::config::get_config_mtime()
    }

    pub fn path() -> &'static std::path::Path {
        crate::configs::config::get_config_path()
    }
}
