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
        pixmap.save_png("/home/i4104/.gemini/antigravity-ide/brain/0a3ae8d8-00e2-403c-a757-f5c7fbd8b467/car_error_preview.png").unwrap();
    }

    #[test]
    fn test_render_full_dashboard() {
        use crate::render::DashboardRenderer;
        let mut renderer = DashboardRenderer::new().unwrap();
        
        // 1. Test Chart mode (tpms_enabled = false) with active speed
        let mut state = VehicleState::default();
        state.tpms_enabled = false;
        state.current_speed = 52.0;
        state.gps_connected = true;
        state.driving_time_secs = 1420.0;
        state.distance_km = 18.5;
        state.max_speed = 88.0;
        state.avg_speed = 46.8;
        
        // Run several ticks so the speed chart advances and stabilizes
        for _ in 0..10 {
            let _ = renderer.render(&state, 0.2);
        }
        renderer
            .pixmap()
            .save_png("/home/i4104/.gemini/antigravity-ide/brain/0a3ae8d8-00e2-403c-a757-f5c7fbd8b467/dashboard_chart_preview.png")
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
            .save_png("/home/i4104/.gemini/antigravity-ide/brain/0a3ae8d8-00e2-403c-a757-f5c7fbd8b467/dashboard_tpms_preview.png")
            .unwrap();
    }
}
