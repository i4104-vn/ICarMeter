// Theme color palette definitions - Pure Black Luxury Dashboard

use tiny_skia::Color;

// Backgrounds & Containers (Pure OLED Pitch Black #000000)
#[inline(always)]
pub fn outer_bg() -> Color {
    Color::from_rgba8(0, 0, 0, 255) // #000000 Pure pitch black
}

#[inline(always)]
pub fn card_bg() -> Color {
    Color::from_rgba8(0, 0, 0, 255) // #000000 Pure black container
}

#[inline(always)]
pub fn card_border() -> Color {
    Color::from_rgba8(38, 38, 42, 255) // #26262A Subtle neutral dark gray border
}

#[inline(always)]
pub fn card_border_glow() -> Color {
    Color::from_rgba8(0, 0, 0, 0) // No blue glow
}

// Section Dividers
#[inline(always)]
pub fn divider() -> Color {
    Color::from_rgba8(28, 28, 32, 255) // #1C1C20 Clean neutral dark divider
}

// Typography
#[inline(always)]
pub fn text_primary() -> Color {
    Color::from_rgba8(255, 255, 255, 255) // #FFFFFF Crisp pure white
}

#[inline(always)]
pub fn text_muted() -> Color {
    Color::from_rgba8(160, 160, 165, 255) // #A0A0A5 Clean neutral light gray (no blue cast)
}

// TPMS & Accents
#[inline(always)]
pub fn accent_cyan() -> Color {
    Color::from_rgba8(210, 215, 220, 255) // #D2D7DC Crisp silver-white accent for leader lines
}

#[inline(always)]
pub fn alert_red() -> Color {
    Color::from_rgba8(235, 35, 35, 255) // #EB2323 Warning red for low tire pressure
}

#[inline(always)]
pub fn alert_red_fill() -> Color {
    Color::from_rgba8(210, 25, 25, 255) // Solid warning red fill for tire
}

#[inline(always)]
pub fn alert_red_glow() -> Color {
    Color::from_rgba8(255, 50, 50, 110) // Ambient red glow for warning tire
}

