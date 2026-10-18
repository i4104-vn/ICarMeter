// Main Dashboard UI Renderer orchestrating panels and converting to pixel buffer

use super::components::{CarPanel, CardContainer, FooterRenderer, SpeedometerPanel, StatsPanel};
use super::core::FontRenderer;
use super::pixel_buffer::PixelBuffer;
use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::models::VehicleState;
use tiny_skia::Pixmap;

pub struct DashboardRenderer {
    pixmap: Pixmap,
    pixel_buffer: PixelBuffer,
    fonts: FontRenderer,
    car_panel: CarPanel,
    speed_panel: SpeedometerPanel,
    stats_panel: StatsPanel,
}

impl DashboardRenderer {
    pub fn new() -> Result<Self, String> {
        let pixmap = Pixmap::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .ok_or_else(|| "Failed to allocate 1280x480 pixmap".to_string())?;
        let fonts = FontRenderer::new()?;
        let car_panel = CarPanel::new();
        let speed_panel = SpeedometerPanel::new();
        let stats_panel = StatsPanel::new();

        Ok(Self {
            pixmap,
            pixel_buffer: PixelBuffer::new(),
            fonts,
            car_panel,
            speed_panel,
            stats_panel,
        })
    }

    pub fn render(&mut self, state: &VehicleState, dt: f32) -> &[u32] {
        // 1. Draw Background, Card Container, and Dividers
        CardContainer::draw(&mut self.pixmap);

        // 2. Render Left Panel (Vector Car + Wheels + TPMS)
        self.car_panel.draw(&mut self.pixmap, &self.fonts, state);

        // 3. Render Center Panel (Speedometer or animated expression companion)
        self.speed_panel
            .draw(&mut self.pixmap, &self.fonts, state, dt, state.current_emo);

        // 4. Render Right Panel (Trip Statistics)
        self.stats_panel.draw(&mut self.pixmap, &self.fonts, state);

        // 5. Render Bottom Footer (Bluetooth, App, GPS status)
        FooterRenderer::draw(&mut self.pixmap, &self.fonts, state);

        self.pixel_buffer.update_from_pixmap(&self.pixmap);

        self.pixel_buffer.as_slice()
    }
}
