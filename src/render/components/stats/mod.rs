// Trip statistics panel facade

pub mod trip_stats;

use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::Pixmap;
use trip_stats::TripStats;

pub struct StatsPanel;

impl StatsPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState) {
        TripStats::draw(pixmap, fonts, state);
    }
}
