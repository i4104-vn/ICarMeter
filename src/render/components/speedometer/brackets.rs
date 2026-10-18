// Speedometer brackets "( )" arcs: progressive bottom-to-top filling based on speed,
// tiered color change upon filling, outward flying animation, and glassmorphic track.

use tiny_skia::{BlendMode, Color, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

pub struct SpeedometerBrackets;

impl SpeedometerBrackets {
    pub fn draw(
        pixmap: &mut Pixmap,
        speed: f32,
        fly_progress: f32,
        alpha: f32,
    ) {
        if alpha <= 0.001 || fly_progress <= 0.001 {
            return;
        }

        let cx = 640.0;
        // Ease-out cubic interpolation for flying brackets
        let t = 1.0 - (1.0 - fly_progress.clamp(0.0, 1.0)).powi(3);

        // Control points ordered strictly from BOTTOM (y = 348.0) to TOP (y = 132.0):
        // Left bracket:
        let l_bottom = (cx + (530.0 - cx) * t, 348.0);
        let l_ctrl_low = (cx + (450.0 - cx) * t, 305.0);
        let l_ctrl_high = (cx + (450.0 - cx) * t, 175.0);
        let l_top = (cx + (530.0 - cx) * t, 132.0);

        // Right bracket:
        let r_bottom = (cx + (750.0 - cx) * t, 348.0);
        let r_ctrl_low = (cx + (830.0 - cx) * t, 305.0);
        let r_ctrl_high = (cx + (830.0 - cx) * t, 175.0);
        let r_top = (cx + (750.0 - cx) * t, 132.0);

        let core_width = 9.5 * t.max(0.4);
        let glow_width = 22.0 * t.max(0.4);

        // 1. Draw Background Track (unlit bracket tube)
        let track_full_left = Self::sub_path(l_bottom, l_ctrl_low, l_ctrl_high, l_top, 1.0);
        let track_full_right = Self::sub_path(r_bottom, r_ctrl_low, r_ctrl_high, r_top, 1.0);

        // Track ambient aura (neutral subtle dark glow)
        let track_aura_color = Color::from_rgba8(50, 50, 55, (20.0 * alpha) as u8);
        Self::stroke_path(pixmap, &track_full_left, track_aura_color, glow_width * 0.85, BlendMode::Screen);
        Self::stroke_path(pixmap, &track_full_right, track_aura_color, glow_width * 0.85, BlendMode::Screen);

        // Track core channel (dark acrylic / unlit neon tube)
        let track_core_color = Color::from_rgba8(28, 28, 32, (190.0 * alpha) as u8);
        Self::stroke_path(pixmap, &track_full_left, track_core_color, core_width, BlendMode::SourceOver);
        Self::stroke_path(pixmap, &track_full_right, track_core_color, core_width, BlendMode::SourceOver);

        // 2. Compute Active Tier (White < 40, Yellow 40..70, Red >= 70):
        // Each tier fills from 0.0 to 1.0 from bottom to top.
        // When a tier is full, crossing into the next tier returns the bar to its initial state
        // and fills the new color from bottom to top!
        let (progress, core_color, glow_color, glow_scale) = if speed < 40.0 {
            // Tier 0: Pure White
            let p = (speed / 40.0).clamp(0.0, 1.0);
            let core = Color::from_rgba8(255, 255, 255, (255.0 * alpha) as u8);
            let glow = Color::from_rgba8(230, 245, 255, (130.0 * alpha) as u8);
            (p, core, glow, 1.0)
        } else if speed < 70.0 {
            // Tier 1: Electric Yellow
            let p = ((speed - 40.0) / 30.0).clamp(0.0, 1.0);
            let core = Color::from_rgba8(255, 215, 0, (255.0 * alpha) as u8);
            let glow = Color::from_rgba8(255, 195, 0, (120.0 * alpha) as u8);
            (p, core, glow, 1.0)
        } else {
            // Tier 2: Orange-Red
            let p = ((speed - 70.0) / 30.0).clamp(0.0, 1.0);
            let core = Color::from_rgba8(255, 69, 0, (255.0 * alpha) as u8);
            let glow = Color::from_rgba8(255, 50, 0, (130.0 * alpha) as u8);
            (p, core, glow, 1.1)
        };

        // 3. Render Active Beam rising from bottom to top in current tier color:
        if progress > 0.001 {
            let left_path = Self::sub_path(l_bottom, l_ctrl_low, l_ctrl_high, l_top, progress);
            let right_path = Self::sub_path(r_bottom, r_ctrl_low, r_ctrl_high, r_top, progress);

            Self::stroke_path(pixmap, &left_path, glow_color, glow_width * glow_scale, BlendMode::Screen);
            Self::stroke_path(pixmap, &right_path, glow_color, glow_width * glow_scale, BlendMode::Screen);
            Self::stroke_path(pixmap, &left_path, core_color, core_width, BlendMode::SourceOver);
            Self::stroke_path(pixmap, &right_path, core_color, core_width, BlendMode::SourceOver);
        }
    }

    /// Trims a cubic Bezier curve from u = 0.0 (bottom) to u = progress (towards top) using De Casteljau's algorithm
    fn sub_path(
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
        progress: f32,
    ) -> Option<Path> {
        let p = progress.clamp(0.0, 1.0);
        if p <= 0.001 {
            return None;
        }

        let (q0, q1, q2, q3) = if p >= 0.999 {
            (p0, p1, p2, p3)
        } else {
            let lerp = |a: (f32, f32), b: (f32, f32), u: f32| (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u);
            let q0 = p0;
            let q1 = lerp(p0, p1, p);
            let r0 = lerp(p1, p2, p);
            let r1 = lerp(p2, p3, p);
            let q2 = lerp(q1, r0, p);
            let s0 = lerp(r0, r1, p);
            let q3 = lerp(q2, s0, p);
            (q0, q1, q2, q3)
        };

        let mut pb = PathBuilder::new();
        pb.move_to(q0.0, q0.1);
        pb.cubic_to(q1.0, q1.1, q2.0, q2.1, q3.0, q3.1);
        pb.finish()
    }

    fn stroke_path(
        pixmap: &mut Pixmap,
        path: &Option<Path>,
        color: Color,
        width: f32,
        blend_mode: BlendMode,
    ) {
        let Some(p) = path else {
            return;
        };
        if color.alpha() == 0.0 {
            return;
        }

        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        paint.blend_mode = blend_mode;

        let stroke = Stroke {
            width,
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: None,
        };

        pixmap.stroke_path(p, &paint, &stroke, Transform::identity(), None);
    }
}
