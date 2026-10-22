//! GPS and speed simulation for exercising the dashboard without live telemetry.

use crate::models::vehicle::state::VehicleState;

const SPEED_CYCLE_SECONDS: f32 = 70.0;
const EARTH_RADIUS_M: f64 = 6_371_000.0;

const HANOI_LOOP: &[(f64, f64)] = &[
    (21.02850, 105.85420),
    (21.02910, 105.85620),
    (21.03135, 105.85735),
    (21.03300, 105.85510),
    (21.03210, 105.85190),
    (21.02960, 105.85010),
    (21.02670, 105.85120),
    (21.02590, 105.85400),
    (21.02700, 105.85615),
];

pub struct DemoGps {
    enabled: bool,
    elapsed_seconds: f32,
    route_distance_m: f64,
}

impl DemoGps {
    pub fn new() -> Self {
        Self {
            enabled: false,
            elapsed_seconds: 0.0,
            route_distance_m: 0.0,
        }
    }

    pub fn toggle(&mut self, state: &mut VehicleState) {
        self.enabled = !self.enabled;
        self.elapsed_seconds = 0.0;
        self.route_distance_m = 0.0;

        if self.enabled {
            let (latitude, longitude) = HANOI_LOOP[0];
            state.gps_connected = true;
            state.gps_latitude = latitude;
            state.gps_longitude = longitude;
            state.gps_heading = bearing(HANOI_LOOP[0], HANOI_LOOP[1]);
            state.current_speed = 0.0;
        } else {
            state.gps_connected = false;
            state.current_speed = 0.0;
        }

        eprintln!(
            "[Demo] Fake GPS {} (press A to toggle)",
            if self.enabled { "enabled" } else { "disabled" }
        );
    }

    pub fn update(&mut self, state: &mut VehicleState, dt: f32) {
        if !self.enabled {
            return;
        }

        let dt = dt.clamp(0.0, 0.1);
        self.elapsed_seconds = (self.elapsed_seconds + dt).rem_euclid(SPEED_CYCLE_SECONDS);
        let speed = speed_at(self.elapsed_seconds);
        self.route_distance_m += f64::from(speed) * f64::from(dt) / 3.6;

        let (latitude, longitude, heading) = position_on_route(self.route_distance_m);
        state.gps_connected = true;
        state.gps_latitude = latitude;
        state.gps_longitude = longitude;
        state.gps_heading = heading;
        state.current_speed = speed;
    }
}

fn speed_at(time: f32) -> f32 {
    match time {
        t if t < 4.0 => smoothstep(t / 4.0) * 22.0,
        t if t < 22.0 => 22.0 + (t * 0.45).sin() * 1.2,
        t if t < 28.0 => (1.0 - smoothstep((t - 22.0) / 6.0)) * 22.0,
        t if t < 34.0 => 0.0,
        t if t < 39.0 => smoothstep((t - 34.0) / 5.0) * 36.0,
        t if t < 54.0 => 36.0 + (t * 0.32).sin() * 1.5,
        t if t < 62.0 => 36.0 - smoothstep((t - 54.0) / 8.0) * 24.0,
        t if t < 66.0 => 12.0,
        t if t < SPEED_CYCLE_SECONDS => (1.0 - smoothstep((t - 66.0) / 4.0)) * 12.0,
        _ => 0.0,
    }
}

fn position_on_route(mut distance_m: f64) -> (f64, f64, f32) {
    let route_length_m: f64 = (0..HANOI_LOOP.len())
        .map(|index| {
            segment_length(
                HANOI_LOOP[index],
                HANOI_LOOP[(index + 1) % HANOI_LOOP.len()],
            )
        })
        .sum();
    distance_m = distance_m.rem_euclid(route_length_m);

    for index in 0..HANOI_LOOP.len() {
        let start = HANOI_LOOP[index];
        let end = HANOI_LOOP[(index + 1) % HANOI_LOOP.len()];
        let segment_length_m = segment_length(start, end);
        if distance_m <= segment_length_m {
            let progress = (distance_m / segment_length_m).clamp(0.0, 1.0);
            let current_bearing = bearing(start, end);
            let previous_bearing = bearing(
                HANOI_LOOP[(index + HANOI_LOOP.len() - 1) % HANOI_LOOP.len()],
                start,
            );
            let next_bearing = bearing(end, HANOI_LOOP[(index + 2) % HANOI_LOOP.len()]);
            let distance_into_segment = progress * segment_length_m;
            let distance_to_turn = segment_length_m - distance_into_segment;
            let heading = if distance_into_segment < 10.0 {
                interpolate_bearing(
                    previous_bearing,
                    current_bearing,
                    distance_into_segment / 10.0,
                )
            } else if distance_to_turn < 10.0 {
                interpolate_bearing(current_bearing, next_bearing, 1.0 - distance_to_turn / 10.0)
            } else {
                current_bearing
            };
            return (
                start.0 + (end.0 - start.0) * progress,
                start.1 + (end.1 - start.1) * progress,
                heading,
            );
        }
        distance_m -= segment_length_m;
    }

    let (latitude, longitude) = HANOI_LOOP[0];
    (latitude, longitude, bearing(HANOI_LOOP[0], HANOI_LOOP[1]))
}

fn segment_length(start: (f64, f64), end: (f64, f64)) -> f64 {
    let mean_latitude = (start.0 + end.0).to_radians() * 0.5;
    let north_m = (end.0 - start.0).to_radians() * EARTH_RADIUS_M;
    let east_m = (end.1 - start.1).to_radians() * EARTH_RADIUS_M * mean_latitude.cos();
    north_m.hypot(east_m)
}

fn bearing(start: (f64, f64), end: (f64, f64)) -> f32 {
    let start_lat = start.0.to_radians();
    let end_lat = end.0.to_radians();
    let delta_lon = (end.1 - start.1).to_radians();
    let y = delta_lon.sin() * end_lat.cos();
    let x = start_lat.cos() * end_lat.sin() - start_lat.sin() * end_lat.cos() * delta_lon.cos();
    y.atan2(x).to_degrees().rem_euclid(360.0) as f32
}

fn interpolate_bearing(from: f32, to: f32, progress: f64) -> f32 {
    let delta = (to - from + 180.0).rem_euclid(360.0) - 180.0;
    (from + delta * progress as f32).rem_euclid(360.0)
}

fn smoothstep(progress: f32) -> f32 {
    let t = progress.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
