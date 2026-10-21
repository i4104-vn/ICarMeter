// Main Dashboard UI Renderer orchestrating panels and converting to pixel buffer

use super::components::{
    CarPanel, CardContainer, FooterRenderer, MapPanel, SpeedometerPanel, StatsPanel,
};
use super::core::FontRenderer;
use super::pixel_buffer::PixelBuffer;
use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::models::VehicleState;
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

const STARTUP_FADE_DURATION: f32 = 1.4;

pub struct DashboardRenderer {
    pixmap: Pixmap,
    pixel_buffer: PixelBuffer,
    fonts: FontRenderer,
    car_panel: CarPanel,
    map_panel: MapPanel,
    speed_panel: SpeedometerPanel,
    stats_panel: StatsPanel,
    footer_panel: FooterRenderer,
    startup_elapsed: f32,
}

impl DashboardRenderer {
    pub fn new() -> Result<Self, String> {
        let pixmap = Pixmap::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .ok_or_else(|| "Failed to allocate 1280x480 pixmap".to_string())?;
        let fonts = FontRenderer::new()?;
        let car_panel = CarPanel::new();
        let map_panel = MapPanel::new();
        let speed_panel = SpeedometerPanel::new();
        let stats_panel = StatsPanel::new();
        let footer_panel = FooterRenderer::new();

        Ok(Self {
            pixmap,
            pixel_buffer: PixelBuffer::new(),
            fonts,
            car_panel,
            map_panel,
            speed_panel,
            stats_panel,
            footer_panel,
            startup_elapsed: 0.0,
        })
    }

    pub fn render(&mut self, state: &VehicleState, dt: f32) -> &[u32] {
        // 1. Draw Background, Card Container, and Dividers
        CardContainer::draw(&mut self.pixmap);

        // 2. Render Left Panel (Vector Car + Wheels + TPMS if enabled, otherwise 2D Offline Map)
        if state.tpms_enabled {
            self.car_panel.draw(&mut self.pixmap, &self.fonts, state);
        } else {
            self.map_panel.draw(&mut self.pixmap, &self.fonts, state);
        }

        // 3. Render Center Panel (Speedometer or animated expression companion)
        self.speed_panel
            .draw(&mut self.pixmap, &self.fonts, state, dt, state.current_emo);

        // 4. Render Right Panel (Trip Statistics)
        self.stats_panel.draw(&mut self.pixmap, &self.fonts, state);

        // 5. Render Bottom Footer with slide-up event ticker in left corner
        self.footer_panel
            .draw(&mut self.pixmap, &self.fonts, state, dt);

        // Fade the complete dashboard in after initialization instead of
        // exposing a fully rendered frame abruptly.
        self.startup_elapsed = (self.startup_elapsed + dt.max(0.0)).min(STARTUP_FADE_DURATION);
        draw_startup_fade(&mut self.pixmap, self.startup_elapsed);

        self.pixel_buffer.update_from_pixmap(&self.pixmap);

        self.pixel_buffer.as_slice()
    }

    #[cfg(test)]
    pub fn pixmap(&self) -> &Pixmap {
        &self.pixmap
    }
}

fn draw_startup_fade(pixmap: &mut Pixmap, elapsed: f32) {
    let progress = (elapsed / STARTUP_FADE_DURATION).clamp(0.0, 1.0);
    let eased = progress * progress * (3.0 - 2.0 * progress);
    let overlay_alpha = ((1.0 - eased) * 255.0).round() as u8;
    if overlay_alpha == 0 {
        return;
    }

    let Some(rect) = Rect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32)
    else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(0, 0, 0, overlay_alpha));
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}
