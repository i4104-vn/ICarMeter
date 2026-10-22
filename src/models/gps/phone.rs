// Mobile companion phone telemetry and GPS data model

use std::time::Instant;

#[allow(dead_code)]
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
