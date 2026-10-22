// Tire Pressure Monitoring System (TPMS) pressures model

pub const LOW_PRESSURE_THRESHOLD: f32 = 2.2;
pub const HIGH_TEMP_THRESHOLD: f32 = 70.0;

#[derive(Debug, Clone, Copy)]
pub struct TirePressures {
    pub fl: f32,                 // Front-Left (bar)
    pub fr: f32,                 // Front-Right (bar)
    pub rl: f32,                 // Rear-Left (bar)
    pub rr: f32,                 // Rear-Right (bar)
    pub fl_temp: f32,            // Front-Left temp (°C)
    pub fr_temp: f32,            // Front-Right temp (°C)
    pub rl_temp: f32,            // Rear-Left temp (°C)
    pub rr_temp: f32,            // Rear-Right temp (°C)
    pub low_pressure_alert: f32, // Alert threshold in bar (default: 2.2)
    pub high_temp_alert: f32,    // Alert threshold in °C (default: 70.0)
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
            low_pressure_alert: LOW_PRESSURE_THRESHOLD,
            high_temp_alert: HIGH_TEMP_THRESHOLD,
        }
    }
}

impl TirePressures {
    #[inline(always)]
    pub fn is_low(&self, pressure: f32) -> bool {
        let threshold = if self.low_pressure_alert > 0.0 {
            self.low_pressure_alert
        } else {
            LOW_PRESSURE_THRESHOLD
        };
        pressure > 0.0 && pressure < threshold
    }

    #[inline(always)]
    pub fn is_high_temp(&self, temp: f32) -> bool {
        let threshold = if self.high_temp_alert > 0.0 {
            self.high_temp_alert
        } else {
            HIGH_TEMP_THRESHOLD
        };
        temp >= threshold
    }

    #[inline(always)]
    pub fn is_disconnected(&self, pressure: f32) -> bool {
        pressure <= 0.0
    }
}
