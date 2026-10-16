// Speed tier solid color definitions (No gradient)

use tiny_skia::Color;

pub fn get_tier_colors(speed: f32) -> (Color, Color) {
    if speed < 40.0 {
        // Trắng (Pure White)
        (
            Color::from_rgba8(255, 255, 255, 255), // Core #FFFFFF
            Color::from_rgba8(220, 240, 255, 120), // Glow
        )
    } else if speed < 70.0 {
        // Vàng (Electric Yellow)
        (
            Color::from_rgba8(255, 215, 0, 255), // Core #FFD700
            Color::from_rgba8(255, 195, 0, 100), // Glow
        )
    } else {
        // Đỏ cam (Orange-Red)
        (
            Color::from_rgba8(255, 69, 0, 255), // Core #FF4500
            Color::from_rgba8(255, 50, 0, 110), // Glow
        )
    }
}
