// Tire Pressure Monitoring System (TPMS) models and telemetry state

pub const LOW_PRESSURE_THRESHOLD: f32 = 2.2; // Under 2.2 bar is flagged as alert

#[derive(Debug, Clone)]
pub struct TirePressures {
    pub fl: f32, // Front-Left
    pub fr: f32, // Front-Right
    pub rl: f32, // Rear-Left
    pub rr: f32, // Rear-Right
}

impl Default for TirePressures {
    fn default() -> Self {
        Self {
            fl: 2.4,
            fr: 2.5,
            rl: 1.9, // Default set to 1.9 bar to demonstrate red alert
            rr: 2.5,
        }
    }
}

impl TirePressures {
    #[inline(always)]
    pub fn is_low(pressure: f32) -> bool {
        pressure < LOW_PRESSURE_THRESHOLD
    }
}
