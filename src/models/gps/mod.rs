//! GPS telemetry received from hardware or the mobile companion.

use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct PhoneGpsData {
    pub spd: Option<f32>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub alt: Option<f32>,
    pub heading: Option<f32>,
    pub last_updated: Option<Instant>,
}

impl PhoneGpsData {
    pub fn new(
        spd: Option<f32>,
        lat: Option<f64>,
        lon: Option<f64>,
        alt: Option<f32>,
        heading: Option<f32>,
    ) -> Self {
        Self {
            spd,
            lat,
            lon,
            alt,
            heading,
            last_updated: Some(Instant::now()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GpsUpdate {
    pub speed_kmh: Option<f32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub heading: Option<f32>,
    pub valid_fix: bool,
}
