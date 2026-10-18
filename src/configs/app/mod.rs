//! Application configuration model and public configuration API.

mod loader;
mod persistence;

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

#[allow(dead_code)]
impl AppConfig {
    pub fn load() -> Self {
        loader::load()
    }

    pub fn to_toml_string(&self) -> String {
        persistence::to_toml(self)
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        persistence::save(self, path)
    }

    pub fn to_json_string(&self) -> String {
        persistence::to_json(self)
    }

    pub fn update_from_json(&mut self, json: &str) -> Result<(), String> {
        persistence::update_from_json(self, json)
    }
}
