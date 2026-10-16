// Main Dashboard UI Renderer orchestrating panels and converting to pixel buffer

use super::components::{CarPanel, CardContainer, SpeedometerPanel, StatsPanel};
use super::core::FontRenderer;
use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::models::VehicleState;
use tiny_skia::Pixmap;

pub struct DashboardRenderer {
    pixmap: Pixmap,
    u32_buffer: Vec<u32>,
    fonts: FontRenderer,
    car_panel: CarPanel,
    speed_panel: SpeedometerPanel,
    stats_panel: StatsPanel,
}

impl DashboardRenderer {
    pub fn new() -> Result<Self, String> {
        let pixmap = Pixmap::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .ok_or_else(|| "Failed to allocate 1280x480 pixmap".to_string())?;
        let u32_buffer = vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT];

        let fonts = FontRenderer::new()?;
        let car_panel = CarPanel::new();
        let speed_panel = SpeedometerPanel::new();
        let stats_panel = StatsPanel::new();

        Ok(Self {
            pixmap,
            u32_buffer,
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

        // 5. Convert Tiny-Skia RGBA bytes to standard 0x00RRGGBB u32 buffer
        let src_bytes = self.pixmap.data();
        let pixel_count = SCREEN_WIDTH * SCREEN_HEIGHT;

        for i in 0..pixel_count {
            let idx = i * 4;
            let r = src_bytes[idx] as u32;
            let g = src_bytes[idx + 1] as u32;
            let b = src_bytes[idx + 2] as u32;
            self.u32_buffer[i] = (r << 16) | (g << 8) | b;
        }

        &self.u32_buffer
    }
}
