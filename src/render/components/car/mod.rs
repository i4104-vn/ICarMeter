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
