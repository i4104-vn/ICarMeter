// Speed tier solid color definitions (No gradient)

use tiny_skia::Color;

/// Determines the solid colors based on speed:
/// - Under 40 km/h : Trắng (Pure White)
/// - Under 70 km/h : Xanh (Cyan / Electric Blue)
/// - Under 95 km/h : Vàng (Electric Yellow)
/// - 95+ km/h      : Đỏ cam (Orange-Red)
pub fn get_tier_colors(speed: f32) -> (Color, Color) {
    if speed < 40.0 {
        // Trắng (Pure White)
        (
            Color::from_rgba8(255, 255, 255, 255), // Core #FFFFFF
            Color::from_rgba8(220, 240, 255, 120), // Glow
        )
    } else if speed < 70.0 {
        // Xanh (Cyan / Electric Blue)
        (
            Color::from_rgba8(0, 220, 255, 255), // Core #00DCFF
            Color::from_rgba8(0, 190, 255, 100), // Glow
        )
    } else if speed < 95.0 {
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
