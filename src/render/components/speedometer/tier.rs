// Speed palette with continuous, smooth transition from Green (calm) to Red (alert).

use tiny_skia::Color;

pub fn get_tier_colors(speed: f32) -> (Color, Color) {
    let speed = speed.clamp(0.0, 120.0);
    let (r, g, b) = if speed < 50.0 {
        // 0 -> 50 km/h: Mượt mà từ Xanh lá (Green) tới Vàng (Yellow)
        let t = speed / 50.0;
        mix_rgb((34.0, 197.0, 94.0), (234.0, 179.0, 8.0), t)
    } else if speed < 90.0 {
        // 50 -> 90 km/h: Mượt mà từ Vàng (Yellow) tới Cam đỏ (Orange-Red)
        let t = (speed - 50.0) / 40.0;
        mix_rgb((234.0, 179.0, 8.0), (249.0, 115.0, 22.0), t)
    } else {
        // 90 -> 120+ km/h: Mượt mà từ Cam đỏ tới Đỏ rực rỡ (Alert Red)
        let t = ((speed - 90.0) / 30.0).min(1.0);
        mix_rgb((249.0, 115.0, 22.0), (239.0, 68.0, 68.0), t)
    };

    let core = Color::from_rgba8(r, g, b, 255);
    let glow = Color::from_rgba8(
        (r as f32 * 0.92).round() as u8,
        (g as f32 * 0.92).round() as u8,
        (b as f32 * 0.92).round() as u8,
        120,
    );
    (core, glow)
}

fn mix_rgb(from: (f32, f32, f32), to: (f32, f32, f32), amount: f32) -> (u8, u8, u8) {
    let t = smoothstep(amount);
    (
        lerp(from.0, to.0, t).round() as u8,
        lerp(from.1, to.1, t).round() as u8,
        lerp(from.2, to.2, t).round() as u8,
    )
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount.clamp(0.0, 1.0)
}

fn smoothstep(amount: f32) -> f32 {
    let t = amount.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
