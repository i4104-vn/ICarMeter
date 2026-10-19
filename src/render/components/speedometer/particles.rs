// Renders straight-line inward suction dots (no tails) with alpha fade

use crate::models::Particle;
use crate::services::particle_engine::PARTICLE_TERMINAL_DISTANCE;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

pub struct ParticleRenderer;

impl ParticleRenderer {
    pub fn draw(
        pixmap: &mut Pixmap,
        particles: &[Particle],
        active_count: usize,
        center_x: f32,
        center_y: f32,
        core_color: Color,
        alpha: f32,
    ) {
        if alpha <= 0.001 {
            return;
        }

        let count = active_count.min(particles.len());

        for p in &particles[..count] {
            let cos_a = p.angle.cos();
            let sin_a = p.angle.sin();
            let px = center_x + cos_a * p.dist;
            let py = center_y + sin_a * (p.dist * 0.78);

            // Keep particles cleanly inside the center section (Dividers at 410.0 and 870.0)
            if px <= 416.0 || px >= 864.0 {
                continue;
            }

            // Boundary edge fade so particles smoothly emerge
            let edge_fade = match px {
                px if px < 436.0 => ((px - 416.0) / 20.0).clamp(0.0, 1.0),
                px if px > 844.0 => ((864.0 - px) / 20.0).clamp(0.0, 1.0),
                _ => 1.0,
            };

            // Keep the center digit area clean. The ellipse follows the
            // visual footprint of the large speed value rather than using a
            // circular cutoff that would leave particles over the glyphs.
            let digit_dx = (px - center_x) / 92.0;
            let digit_dy = (py - (center_y + 8.0)) / 106.0;
            if digit_dx * digit_dx + digit_dy * digit_dy < 1.0 {
                continue;
            }

            // Fade particles as they reach the inner edge of the brackets.
            // They disappear before entering the digit-safe zone.
            let inner_fade = ((p.dist - PARTICLE_TERMINAL_DISTANCE) / 28.0).clamp(0.0, 1.0);

            let combined_alpha = (edge_fade * inner_fade * alpha * 255.0) as u8;
            if combined_alpha == 0 {
                continue;
            }

            let mut dot_paint = Paint::default();
            dot_paint.set_color(Color::from_rgba8(
                (core_color.red() * 255.0) as u8,
                (core_color.green() * 255.0) as u8,
                (core_color.blue() * 255.0) as u8,
                combined_alpha,
            ));
            dot_paint.anti_alias = true;

            // Draw clean circular dot (No tail)
            let mut pb = PathBuilder::new();
            pb.push_circle(px, py, p.size);
            if let Some(path) = pb.finish() {
                pixmap.fill_path(
                    &path,
                    &dot_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }
    }
}
