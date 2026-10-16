// Vehicle physics and automatic telemetry simulation service

use crate::models::VehicleState;

pub struct VehicleSimulator;

impl VehicleSimulator {
    pub fn update(state: &mut VehicleState, dt: f32) {
        state.sim_time += dt;

        // Realistic driving scenario loop (65s cycle):
        // - 0s..16s: Cruising at city/highway speeds (35 - 75 km/h)
        // - 16s..19s: Smooth deceleration to full stop (0 km/h)
        // - 19s..52s: Stationary idle at 0 km/h (33 seconds idle!)
        //      * 19s..22s: Wait 3s at speed 0
        //      * 22s..34s: Emo companion active in Mischievous mode (Cheeky, Cat, Squint, Sneaky, Surprise)
        //      * 34s..37.5s: Yawn (Ngáp ngủ)
        //      * 37.5s..42s: Drowsy (Mắt lim dim)
        //      * 42s..52s: Deep Sleep (Ngủ say with floating z Z Z)
        // - 52s..65s: Smooth acceleration! (Wakes up, Emo fades, brackets fly out, speed digits appear!)
        let cycle = state.sim_time % 65.0;

        let target_speed = if cycle < 16.0 {
            54.0 + (cycle * 0.45).sin() * 20.0
        } else if cycle < 52.0 {
            0.0
        } else {
            let accel_phase = (cycle - 52.0) / 13.0;
            accel_phase * 68.0
        };

        // Smooth physical inertia
        let speed_diff = target_speed - state.current_speed;
        let lerp_factor = if target_speed < 0.5 {
            dt * 4.0
        } else {
            dt * 2.2
        };
        state.current_speed += speed_diff * lerp_factor.min(1.0);

        if state.current_speed < 0.15 && target_speed == 0.0 {
            state.current_speed = 0.0;
        }

        // Update odometer and session timer automatically
        if state.current_speed > 0.5 {
            state.driving_time_secs += dt as f64;

            let distance_increment = (state.current_speed / 3600.0) * dt;
            state.distance_km += distance_increment;

            if state.current_speed > state.max_speed {
                state.max_speed = state.current_speed;
            }

            if state.driving_time_secs > 1.0 {
                let hours = (state.driving_time_secs / 3600.0) as f32;
                state.avg_speed = state.distance_km / hours;
            }
        }
    }
}
