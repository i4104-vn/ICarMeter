// High-performance text rendering using Fontdue and Tiny-Skia

use fontdue::{Font, FontSettings};
use tiny_skia::{Color, Pixmap};

pub struct FontRenderer {
    pub regular: Font,
    pub bold: Font,
}

impl FontRenderer {
    pub fn new() -> Result<Self, String> {
        // Embed high quality fonts so the binary is 100% standalone
        const FONT_REGULAR_BYTES: &[u8] =
            include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
        const FONT_BOLD_BYTES: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Bold.ttf");

        let regular = Font::from_bytes(FONT_REGULAR_BYTES, FontSettings::default())
            .map_err(|e| format!("Failed to parse regular font: {}", e))?;
        let bold = Font::from_bytes(FONT_BOLD_BYTES, FontSettings::default())
            .map_err(|e| format!("Failed to parse bold font: {}", e))?;

        Ok(Self { regular, bold })
    }

    /// Measures the horizontal width of text in pixels
    pub fn measure_width(&self, font: &Font, text: &str, px_size: f32) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            let metrics = font.metrics(ch, px_size);
            width += metrics.advance_width;
        }
        width
    }

    /// Draws solid colored text onto the pixmap
    pub fn draw_text(
        &self,
        pixmap: &mut Pixmap,
        font: &Font,
        text: &str,
        x: f32,
        baseline_y: f32,
        px_size: f32,
        color: Color,
    ) {
        let mut cur_x = x;
        let p_width = pixmap.width() as i32;
        let p_height = pixmap.height() as i32;
        let data = pixmap.data_mut();

        let sr = (color.red() * 255.0) as u32;
        let sg = (color.green() * 255.0) as u32;
        let sb = (color.blue() * 255.0) as u32;
        let sa = (color.alpha() * 255.0) as u32;

        for ch in text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, px_size);

            let gx = (cur_x + metrics.xmin as f32).round() as i32;
            let gy = (baseline_y - metrics.height as f32 - metrics.ymin as f32).round() as i32;

            for row in 0..metrics.height {
                let py = gy + row as i32;
                if py < 0 || py >= p_height {
                    continue;
                }

                for col in 0..metrics.width {
                    let px = gx + col as i32;
                    if px < 0 || px >= p_width {
                        continue;
                    }

                    let alpha_coverage = bitmap[row * metrics.width + col] as u32;
                    if alpha_coverage == 0 {
                        continue;
                    }

                    let blended_alpha = (sa * alpha_coverage) / 255;
                    if blended_alpha == 0 {
                        continue;
                    }

                    let pixel_idx = ((py as usize * p_width as usize) + px as usize) * 4;
                    let dr = data[pixel_idx] as u32;
                    let dg = data[pixel_idx + 1] as u32;
                    let db = data[pixel_idx + 2] as u32;
                    let da = data[pixel_idx + 3] as u32;

                    // Premultiplied alpha blending
                    let inv_a = 255 - blended_alpha;
                    let premul_r = (sr * blended_alpha) / 255;
                    let premul_g = (sg * blended_alpha) / 255;
                    let premul_b = (sb * blended_alpha) / 255;

                    data[pixel_idx] = (premul_r + (dr * inv_a) / 255).min(255) as u8;
                    data[pixel_idx + 1] = (premul_g + (dg * inv_a) / 255).min(255) as u8;
                    data[pixel_idx + 2] = (premul_b + (db * inv_a) / 255).min(255) as u8;
                    data[pixel_idx + 3] = (blended_alpha + (da * inv_a) / 255).min(255) as u8;
                }
            }

            cur_x += metrics.advance_width;
        }
    }
}
