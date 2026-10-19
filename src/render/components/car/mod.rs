// Car panel facade: coordinates car chassis, wheels, and TPMS overlay

pub mod chassis;
pub mod tpms;
pub mod wheels;

use crate::models::VehicleState;
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
        let state = VehicleState::default(); // default tires are all 0.0 (disconnected / error)
        let panel = CarPanel::new();
        panel.draw(&mut pixmap, &fonts, &state);
        pixmap.save_png("/home/i4104/.gemini/antigravity-ide/brain/be3d16d5-a5f8-4926-bf08-b4eebe1ec757/car_error_preview.png").unwrap();
    }

    #[test]
    fn test_render_full_dashboard() {
        use crate::render::DashboardRenderer;
        let mut renderer = DashboardRenderer::new().unwrap();
        let state = VehicleState::default();
        let _ = renderer.render(&state, 2.0); // complete startup fade
        renderer
            .pixmap()
            .save_png("/home/i4104/.gemini/antigravity-ide/brain/be3d16d5-a5f8-4926-bf08-b4eebe1ec757/full_dashboard_preview.png")
            .unwrap();
    }
}
