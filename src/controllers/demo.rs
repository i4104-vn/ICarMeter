//! Visual demo controls for testing the dashboard without live vehicle data.

use crate::models::VehicleState;

const DEMO_CYCLE_SECONDS: f32 = 10.0;
const DEMO_ACCELERATION_SECONDS: f32 = 3.5;
const DEMO_DECELERATION_SECONDS: f32 = 3.5;

pub struct DemoSpeed {
    enabled: bool,
    elapsed: f32,
}

impl DemoSpeed {
    pub fn new() -> Self {
        Self {
            enabled: false,
            elapsed: 0.0,
        }
    }

    pub fn toggle(&mut self, state: &mut VehicleState) {
        self.enabled = !self.enabled;
        self.elapsed = 0.0;

        if !self.enabled {
            // Let the next telemetry update provide a real value. Clearing it
            // here also prevents a stale demo speed when GPS is unavailable.
            state.current_speed = 0.0;
        }

        eprintln!(
            "[Demo] Fake speed {} (press A to toggle)",
            if self.enabled { "enabled" } else { "disabled" }
        );
    }

    pub fn update(&mut self, state: &mut VehicleState, dt: f32) {
        if !self.enabled {
            return;
        }

        self.elapsed = (self.elapsed + dt.max(0.0)).rem_euclid(DEMO_CYCLE_SECONDS);
        state.current_speed = speed_at(self.elapsed);
    }
}

fn speed_at(time: f32) -> f32 {
    if time < DEMO_ACCELERATION_SECONDS {
        let progress = time / DEMO_ACCELERATION_SECONDS;
        smoothstep(progress) * 100.0
    } else if time < DEMO_ACCELERATION_SECONDS + DEMO_DECELERATION_SECONDS {
        let progress = (time - DEMO_ACCELERATION_SECONDS) / DEMO_DECELERATION_SECONDS;
        (1.0 - smoothstep(progress)) * 100.0
    } else {
        0.0
    }
}

fn smoothstep(progress: f32) -> f32 {
    let t = progress.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
