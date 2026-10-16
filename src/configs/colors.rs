// Theme color palette definitions

use tiny_skia::Color;

// Backgrounds & Containers
#[inline(always)]
pub fn outer_bg() -> Color {
    Color::from_rgba8(6, 16, 25, 255) // #061019 Deep dark navy/black
}

#[inline(always)]
pub fn card_bg() -> Color {
    Color::from_rgba8(0, 8, 13, 255) // #00080D Inner card background
}

#[inline(always)]
pub fn card_border() -> Color {
    Color::from_rgba8(24, 46, 66, 255) // #182E42 Border stroke
}

#[inline(always)]
pub fn card_border_glow() -> Color {
    Color::from_rgba8(14, 52, 85, 80) // Border ambient glow
}

// Section Dividers
#[inline(always)]
pub fn divider() -> Color {
    Color::from_rgba8(13, 30, 46, 255) // #0D1E2E
}

// Typography
#[inline(always)]
pub fn text_primary() -> Color {
    Color::from_rgba8(255, 255, 255, 255) // #FFFFFF Crisp white
}

#[inline(always)]
pub fn text_muted() -> Color {
    Color::from_rgba8(140, 163, 184, 255) // #8CA3B8 Muted grayish blue
}

// TPMS & Accents
#[inline(always)]
pub fn accent_cyan() -> Color {
    Color::from_rgba8(88, 180, 248, 255) // #58B4F8 Normal TPMS unit & leader lines
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
