// Speedometer brackets "( )" arcs with solid tier color, neon glow, and outward flying animation

use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct SpeedometerBrackets;

impl SpeedometerBrackets {
    pub fn draw(
        pixmap: &mut Pixmap,
        core_color: Color,
        glow_color: Color,
        fly_progress: f32,
        alpha: f32,
    ) {
        if alpha <= 0.001 || fly_progress <= 0.001 {
            return;
        }

        let cx = 640.0;
        // Ease-out cubic interpolation
        let t = 1.0 - (1.0 - fly_progress.clamp(0.0, 1.0)).powi(3);
        let a = (alpha * 255.0).clamp(0.0, 255.0) as u8;
        let glow_a = ((alpha * glow_color.alpha() * 255.0) as u8).min(255);

        // Left bracket: interpolate from cx=640.0 outwards to normal coordinates
        let l_start_x = cx + (530.0 - cx) * t;
        let l_ctrl1_x = cx + (450.0 - cx) * t;
        let l_ctrl2_x = cx + (450.0 - cx) * t;
        let l_end_x = cx + (530.0 - cx) * t;

        let mut pb_left = PathBuilder::new();
        pb_left.move_to(l_start_x, 132.0);
        pb_left.cubic_to(l_ctrl1_x, 175.0, l_ctrl2_x, 305.0, l_end_x, 348.0);
        let left_path = pb_left.finish();

        // Right bracket: interpolate from cx=640.0 outwards to normal coordinates
        let r_start_x = cx + (750.0 - cx) * t;
        let r_ctrl1_x = cx + (830.0 - cx) * t;
        let r_ctrl2_x = cx + (830.0 - cx) * t;
        let r_end_x = cx + (750.0 - cx) * t;

        let mut pb_right = PathBuilder::new();
        pb_right.move_to(r_start_x, 132.0);
        pb_right.cubic_to(r_ctrl1_x, 175.0, r_ctrl2_x, 305.0, r_end_x, 348.0);
        let right_path = pb_right.finish();

        // Outer Glow Pass
        let mut glow_paint = Paint::default();
        glow_paint.set_color(Color::from_rgba8(
            (glow_color.red() * 255.0) as u8,
            (glow_color.green() * 255.0) as u8,
            (glow_color.blue() * 255.0) as u8,
            glow_a,
        ));
        glow_paint.anti_alias = true;
        glow_paint.blend_mode = tiny_skia::BlendMode::Screen;

        let glow_stroke = Stroke {
            width: 22.0 * t.max(0.4),
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: None,
        };

        if let Some(ref p) = left_path {
            pixmap.stroke_path(p, &glow_paint, &glow_stroke, Transform::identity(), None);
        }
        if let Some(ref p) = right_path {
            pixmap.stroke_path(p, &glow_paint, &glow_stroke, Transform::identity(), None);
        }

        // Core Solid Stroke Pass
        let mut core_paint = Paint::default();
        core_paint.set_color(Color::from_rgba8(
            (core_color.red() * 255.0) as u8,
            (core_color.green() * 255.0) as u8,
            (core_color.blue() * 255.0) as u8,
            a,
        ));
        core_paint.anti_alias = true;

        let core_stroke = Stroke {
            width: 9.5 * t.max(0.4),
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: None,
        };

        if let Some(ref p) = left_path {
            pixmap.stroke_path(p, &core_paint, &core_stroke, Transform::identity(), None);
        }
        if let Some(ref p) = right_path {
            pixmap.stroke_path(p, &core_paint, &core_stroke, Transform::identity(), None);
        }
    }
}
