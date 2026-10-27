// Car panel facade: coordinates car chassis, wheels, and TPMS overlay

pub mod chassis;
pub mod tpms;
pub mod wheels;

use crate::models::vehicle::state::VehicleState;
use crate::render::core::FontRenderer;
use chassis::CarChassis;
use tiny_skia::Pixmap;
use tpms::TpmsOverlay;
use wheels::Wheels;

pub struct CarPanel;

impl CarPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState) {
        let cx = 222.0;
        let cy = 240.0;

        // 1. Draw 4 side wheels (highlights low tire pressure in red)
        Wheels::draw(pixmap, cx, cy, &state.tires);

        // 2. Draw Vector Car Chassis Body
        CarChassis::draw(pixmap, cx, cy);

        // 3. Draw 4 Tire Pressure Indicators & Leader Lines
        TpmsOverlay::draw(pixmap, fonts, cx, cy, &state.tires);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_car_error() {
        let mut pixmap = Pixmap::new(440, 480).unwrap();
        pixmap.fill(tiny_skia::Color::BLACK);
        let fonts = FontRenderer::new().unwrap();
        let state = VehicleState::default();
        let panel = CarPanel::new();
        panel.draw(&mut pixmap, &fonts, &state);
        pixmap.save_png("../../car_error_preview.png").unwrap();
    }

    #[test]
    fn test_render_full_dashboard() {
        use crate::render::DashboardRenderer;
        let mut renderer = DashboardRenderer::new().unwrap();
        
        // 1. Test Chart mode (tpms_enabled = false) with realistic acceleration curve
        let mut state = VehicleState::default();
        state.tpms_enabled = false;
        state.gps_connected = true;
        state.driving_time_secs = 1420.0;
        state.distance_km = 18.5;
        state.max_speed = 88.0;
        state.avg_speed = 46.8;
        
        // Simulate 4 seconds of driving at 60 FPS (accel 0 -> 65 km/h, cruise, slight curve)
        let total_frames = 240;
        for frame in 0..total_frames {
            let t = frame as f32 / 60.0;
            state.current_speed = if t < 1.0 {
                0.0
            } else if t < 2.5 {
                let p = (t - 1.0) / 1.5;
                p * p * (3.0 - 2.0 * p) * 65.0
            } else {
                65.0 + ((t - 2.5) * 2.0).sin() * 5.0
            };
            let _ = renderer.render(&state, 1.0 / 60.0);
        }
        renderer
            .pixmap()
            .save_png("dashboard_chart_preview.png")
            .unwrap();

        // 2. Test TPMS mode (tpms_enabled = true)
        state.tpms_enabled = true;
        state.tires.fl = 2.3;
        state.tires.fr = 2.4;
        state.tires.rl = 2.3;
        state.tires.rr = 2.2;
        state.tires.fl_temp = 32.0;
        state.tires.fr_temp = 34.0;
        state.tires.rl_temp = 31.0;
        state.tires.rr_temp = 30.0;
        let _ = renderer.render(&state, 0.1);
        renderer
            .pixmap()
            .save_png("dashboard_tpms_preview.png")
            .unwrap();
    }
}
