// Vehicle status and telemetry data model

use super::emotion::EmoExpression;
use super::tpms::TirePressures;

#[derive(Debug, Clone)]
pub struct VehicleState {
    pub current_speed: f32,
    pub driving_time_secs: f64,
    pub distance_km: f32,
    pub max_speed: f32,
    pub avg_speed: f32,
    pub tires: TirePressures,
    pub current_emo: Option<EmoExpression>,
    pub bt_active: bool,
    pub bt_device_name: Option<String>,
    pub gps_connected: bool,
    pub tpms_enabled: bool,
    pub show_bt_log: bool,
    pub bt_logs: Vec<String>,
    pub gps_latitude: f64,
    pub gps_longitude: f64,
    pub gps_heading: f32,
    pub gps_altitude: f32,
}

impl Default for VehicleState {
    fn default() -> Self {
        Self {
            current_speed: 0.0,
            driving_time_secs: 0.0,
            distance_km: 0.0,
            max_speed: 0.0,
            avg_speed: 0.0,
            tires: TirePressures::default(),
            current_emo: None,
            bt_active: true,
            bt_device_name: None,
            gps_connected: false,
            tpms_enabled: true,
            show_bt_log: false,
            bt_logs: Vec::new(),
            gps_latitude: 21.0285,
            gps_longitude: 105.8542,
            gps_heading: 0.0,
            gps_altitude: 18.0,
        }
    }
}

impl VehicleState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_expression_name(&mut self, name: &str) {
        self.current_emo = EmoExpression::from_str(name);
    }

    pub fn update_trip(&mut self, dt: f32) {
        if self.current_speed > 0.5 {
            self.driving_time_secs += dt as f64;
            let distance_increment = (self.current_speed / 3600.0) * dt;
            self.distance_km += distance_increment;

            if self.current_speed > self.max_speed {
                self.max_speed = self.current_speed;
            }

            if self.driving_time_secs > 1.0 {
                let hours = (self.driving_time_secs / 3600.0) as f32;
                self.avg_speed = self.distance_km / hours;
            }
        }
    }

    pub fn formatted_driving_time(&self) -> String {
        let total_secs = self.driving_time_secs as u64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    pub fn formatted_distance(&self) -> String {
        format!("{:.1} km", self.distance_km)
    }

    pub fn formatted_max_speed(&self) -> String {
        format!("{:.0} km/h", self.max_speed.round())
    }

    pub fn formatted_avg_speed(&self) -> String {
        format!("{:.0} km/h", self.avg_speed.round())
    }
}
