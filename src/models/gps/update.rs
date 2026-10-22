// NMEA GPS hardware telemetry update model

#[derive(Debug, Clone, Copy, Default)]
pub struct GpsUpdate {
    pub speed_kmh: Option<f32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub heading: Option<f32>,
    pub valid_fix: bool,
}
