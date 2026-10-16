// Vehicle status and telemetry data model

use super::tpms::TirePressures;

#[derive(Debug, Clone)]
pub struct VehicleState {
    pub current_speed: f32,
    pub driving_time_secs: f64,
    pub distance_km: f32,
    pub max_speed: f32,
    pub avg_speed: f32,
    pub tires: TirePressures,
    pub sim_time: f32,
}

impl Default for VehicleState {
    fn default() -> Self {
        Self {
            current_speed: 68.0,
            driving_time_secs: 5076.0, // 01:24:36
            distance_km: 78.6,
            max_speed: 112.0,
            avg_speed: 56.0,
            tires: TirePressures::default(),
            sim_time: 0.0,
        }
    }
}

impl VehicleState {
    pub fn new() -> Self {
        Self::default()
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
