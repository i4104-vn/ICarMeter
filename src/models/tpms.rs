// Tire Pressure Monitoring System (TPMS) models and telemetry state

pub const LOW_PRESSURE_THRESHOLD: f32 = 2.2;

#[derive(Debug, Clone, Copy)]
pub struct TirePressures {
    pub fl: f32, // Front-Left
    pub fr: f32, // Front-Right
    pub rl: f32, // Rear-Left
    pub rr: f32, // Rear-Right
}

impl Default for TirePressures {
    fn default() -> Self {
        Self {
            fl: 0.0,
            fr: 0.0,
            rl: 0.0,
            rr: 0.0,
        }
    }
}

impl TirePressures {
    #[inline(always)]
    pub fn is_low(pressure: f32) -> bool {
        pressure > 0.0 && pressure < LOW_PRESSURE_THRESHOLD
    }
}
