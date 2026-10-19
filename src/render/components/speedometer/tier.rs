// Speed palette with smooth transitions between calm, cruising and alert tones.

use tiny_skia::Color;

pub fn get_tier_colors(speed: f32) -> (Color, Color) {
    let speed = speed.clamp(0.0, 100.0);
    let (r, g, b) = match speed {
        speed if speed < 42.0 => {
            // Icy silver -> clean cyan: calm but still technical.
            mix_rgb((220.0, 235.0, 242.0), (74.0, 222.0, 218.0), speed / 42.0)
        }
        speed if speed < 72.0 => {
            // Cyan -> warm amber: a gradual cruising-to-alert transition.
            mix_rgb(
                (74.0, 222.0, 218.0),
                (255.0, 193.0, 92.0),
                (speed - 42.0) / 30.0,
            )
        }
        speed => {
            // Amber -> coral red: reserve the warning tone for genuinely high speed.
            mix_rgb(
                (255.0, 193.0, 92.0),
                (255.0, 86.0, 92.0),
                (speed - 72.0) / 28.0,
            )
        }
    };

    let core = Color::from_rgba8(r, g, b, 255);
    let glow = Color::from_rgba8(
        (r as f32 * 0.92).round() as u8,
        (g as f32 * 0.92).round() as u8,
        (b as f32 * 0.92).round() as u8,
        118,
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
