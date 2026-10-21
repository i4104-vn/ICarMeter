// Colors used by the map panel outside the MapLibre style itself.

use tiny_skia::Color;

/// Fallback and panel colors around the rendered MapLibre map.
#[derive(Debug, Clone)]
pub struct MapAppearance {
    pub background_color: Color,
    pub card_border: Color,
}

impl Default for MapAppearance {
    fn default() -> Self {
        Self {
            background_color: Color::from_rgba8(8, 12, 20, 255),
            card_border: Color::from_rgba8(30, 41, 59, 220),
        }
    }
}
