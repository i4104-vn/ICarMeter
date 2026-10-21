// MapLibre GL Style Specification (v8) definitions and Dark Navigation Theme
// Conforms to MapLibre GL Style Specification concepts:
// https://maplibre.org/maplibre-style-spec/

use tiny_skia::Color;

/// MapLibre Layer Type
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    Background,
    Fill,
    Line,
    Symbol,
}

/// Paint properties for a vector line layer
#[derive(Debug, Clone)]
pub struct LinePaint {
    pub color: Color,
    pub width: f32,
    #[allow(dead_code)]
    pub opacity: f32,
}

/// Paint properties for a vector fill layer
#[derive(Debug, Clone)]
pub struct FillPaint {
    pub fill_color: Color,
    pub outline_color: Option<Color>,
    pub outline_width: f32,
}

/// Complete MapLibre GL Style configuration for the 2D offline dashboard
#[derive(Debug, Clone)]
pub struct MapLibreStyle {
    #[allow(dead_code)]
    pub name: &'static str,
    #[allow(dead_code)]
    pub version: u8,
    pub background_color: Color,
    pub grid_color: Color,
    pub water_fill: FillPaint,
    pub building_fill: FillPaint,
    pub road_secondary: LinePaint,
    pub route_glow: LinePaint,
    pub route_line: LinePaint,
    pub route_core: LinePaint,
    pub waypoint_color: Color,
    pub arrow_color: Color,
    pub arrow_stroke: Color,
    pub arrow_halo: Color,
    pub card_border: Color,
    pub hud_bg: Color,
    pub hud_border: Color,
    pub hud_text_primary: Color,
    pub hud_text_muted: Color,
    pub hud_fix_dot: Color,
}

impl MapLibreStyle {
    /// Curated MapLibre GL Dark Navigation vector theme
    /// High-contrast tactical palette optimized for night driving and OLED/LCD displays
    pub fn dark_navigation() -> Self {
        Self {
            name: "MapLibre Dark Navigation v8",
            version: 8,
            // Canvas background: ultra-dark navy slate #080c14
            background_color: Color::from_rgba8(8, 12, 20, 255),
            // Tactical coordinate grid: subtle #101828 with alpha
            grid_color: Color::from_rgba8(16, 24, 40, 180),
            // Water bodies: deep midnight navy with royal blue edge
            water_fill: FillPaint {
                fill_color: Color::from_rgba8(12, 22, 38, 230),
                outline_color: Some(Color::from_rgba8(29, 78, 216, 90)),
                outline_width: 1.5,
            },
            // Building footprints: dark graphite blocks
            building_fill: FillPaint {
                fill_color: Color::from_rgba8(11, 17, 28, 220),
                outline_color: Some(Color::from_rgba8(20, 30, 48, 140)),
                outline_width: 1.0,
            },
            // Secondary roads: muted slate arterials
            road_secondary: LinePaint {
                color: Color::from_rgba8(30, 41, 59, 190),
                width: 2.2,
                opacity: 0.75,
            },
            // Primary route outer glow: MapLibre cobalt aura
            route_glow: LinePaint {
                color: Color::from_rgba8(37, 99, 235, 110),
                width: 8.0,
                opacity: 0.43,
            },
            // Primary route line: vibrant electric cyan
            route_line: LinePaint {
                color: Color::from_rgba8(56, 189, 248, 240),
                width: 3.4,
                opacity: 0.94,
            },
            // Primary route inner core: crisp white
            route_core: LinePaint {
                color: Color::WHITE,
                width: 1.2,
                opacity: 1.0,
            },
            // Waypoint diamond markers
            waypoint_color: Color::from_rgba8(56, 189, 248, 220),
            // Vehicle navigation puck / arrow
            arrow_color: Color::from_rgba8(56, 189, 248, 255),
            arrow_stroke: Color::WHITE,
            arrow_halo: Color::from_rgba8(2, 132, 199, 90),
            // UI Panels and HUD elements
            card_border: Color::from_rgba8(30, 41, 59, 220),
            hud_bg: Color::from_rgba8(15, 23, 42, 220),
            hud_border: Color::from_rgba8(51, 65, 85, 180),
            hud_text_primary: Color::from_rgba8(56, 189, 248, 255),
            hud_text_muted: Color::from_rgba8(148, 163, 184, 230),
            hud_fix_dot: Color::from_rgba8(34, 197, 94, 255),
        }
    }
}
