// Speedometer digits rendering with solid tier color and alpha fade

use crate::render::core::FontRenderer;
use tiny_skia::{Color, Pixmap};

pub struct SpeedometerDigits;

impl SpeedometerDigits {
    pub fn draw(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        speed: f32,
        center_x: f32,
        core_color: Color,
        alpha: f32,
    ) {
        if alpha <= 0.001 {
            return;
        }

        let speed_val = speed.round() as i32;
        let speed_str = format!("{}", speed_val);

        let font_size = 156.0;
        let text_w = fonts.measure_width(&fonts.bold, &speed_str, font_size);
        let text_x = center_x - (text_w / 2.0);
        let baseline_y = 294.0;

        let a = (alpha * 255.0).clamp(0.0, 255.0) as u8;
        let tinted_color = Color::from_rgba8(
            (core_color.red() * 255.0) as u8,
            (core_color.green() * 255.0) as u8,
            (core_color.blue() * 255.0) as u8,
            a,
        );

        fonts.draw_text(
            pixmap,
            &fonts.bold,
            &speed_str,
            text_x,
            baseline_y,
            font_size,
            tinted_color,
        );
    }
}
