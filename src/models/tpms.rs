// Tire Pressure Monitoring System (TPMS) models and telemetry state

pub const LOW_PRESSURE_THRESHOLD: f32 = 2.2;
pub const HIGH_TEMP_THRESHOLD: f32 = 70.0;

#[derive(Debug, Clone, Copy)]
pub struct TirePressures {
    pub fl: f32,      // Front-Left (bar)
    pub fr: f32,      // Front-Right (bar)
    pub rl: f32,      // Rear-Left (bar)
    pub rr: f32,      // Rear-Right (bar)
    pub fl_temp: f32, // Front-Left temp (°C)
    pub fr_temp: f32, // Front-Right temp (°C)
    pub rl_temp: f32, // Rear-Left temp (°C)
    pub rr_temp: f32, // Rear-Right temp (°C)
}

impl Default for TirePressures {
    fn default() -> Self {
        Self {
            fl: 0.0,
            fr: 0.0,
            rl: 0.0,
            rr: 0.0,
            fl_temp: 0.0,
            fr_temp: 0.0,
            rl_temp: 0.0,
            rr_temp: 0.0,
        }
    }
}

impl TirePressures {
    #[inline(always)]
    pub fn is_low(pressure: f32) -> bool {
        pressure > 0.0 && pressure < LOW_PRESSURE_THRESHOLD
    }

    #[inline(always)]
    pub fn is_high_temp(temp: f32) -> bool {
        temp >= HIGH_TEMP_THRESHOLD
    }
}
