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

        // Track ambient aura
        let track_aura_color = Color::from_rgba8(0, 120, 170, (25.0 * alpha) as u8);
        Self::stroke_path(pixmap, &track_full_left, track_aura_color, glow_width * 0.85, BlendMode::Screen);
        Self::stroke_path(pixmap, &track_full_right, track_aura_color, glow_width * 0.85, BlendMode::Screen);

        // Track core channel (dark acrylic / unlit neon tube)
        let track_core_color = Color::from_rgba8(16, 34, 48, (190.0 * alpha) as u8);
        Self::stroke_path(pixmap, &track_full_left, track_core_color, core_width, BlendMode::SourceOver);
        Self::stroke_path(pixmap, &track_full_right, track_core_color, core_width, BlendMode::SourceOver);

        // 2. Compute Tier Fill Progresses from Bottom to Top:
        // Tier 0 (White): 0.0 -> 40.0 km/h (fills from bottom to top with pure White)
        // Tier 1 (Cyan): 40.0 -> 70.0 km/h (fills over full White bar)
        // Tier 2 (Yellow): 70.0 -> 95.0 km/h (fills over full Cyan bar)
        // Tier 3 (Orange-Red): 95.0 -> 120.0+ km/h (fills over full Yellow bar)
        let p_white = (speed / 40.0).clamp(0.0, 1.0);
        let p_cyan = if speed > 40.0 {
            ((speed - 40.0) / 30.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let p_yellow = if speed > 70.0 {
            ((speed - 70.0) / 25.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let p_red = if speed > 95.0 {
            ((speed - 95.0) / 25.0).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Tier Color Palettes:
        // Tier 0: Pure White (#FFFFFF)
        let white_core = Color::from_rgba8(255, 255, 255, (255.0 * alpha) as u8);
        let white_glow = Color::from_rgba8(230, 245, 255, (130.0 * alpha) as u8);

        // Tier 1: Cyan / Electric Blue (#00DCFF)
        let cyan_core = Color::from_rgba8(0, 220, 255, (255.0 * alpha) as u8);
        let cyan_glow = Color::from_rgba8(0, 190, 255, (120.0 * alpha) as u8);

        // Tier 2: Electric Yellow / Gold (#FFD700)
        let yellow_core = Color::from_rgba8(255, 215, 0, (255.0 * alpha) as u8);
        let yellow_glow = Color::from_rgba8(255, 195, 0, (120.0 * alpha) as u8);

        // Tier 3: Orange-Red / Crimson (#FF4500)
        let red_core = Color::from_rgba8(255, 69, 0, (255.0 * alpha) as u8);
        let red_glow = Color::from_rgba8(255, 50, 0, (130.0 * alpha) as u8);

        // 3. Render Active Tier Beams:
        // 1) White beam: fills from bottom to top up to 40 km/h (reaches 100% full White at 40 km/h)
        if p_white > 0.001 {
            let left_w = Self::sub_path(l_bottom, l_ctrl_low, l_ctrl_high, l_top, p_white);
            let right_w = Self::sub_path(r_bottom, r_ctrl_low, r_ctrl_high, r_top, p_white);
            Self::stroke_path(pixmap, &left_w, white_glow, glow_width, BlendMode::Screen);
            Self::stroke_path(pixmap, &right_w, white_glow, glow_width, BlendMode::Screen);
            Self::stroke_path(pixmap, &left_w, white_core, core_width, BlendMode::SourceOver);
            Self::stroke_path(pixmap, &right_w, white_core, core_width, BlendMode::SourceOver);
        }

        // 2) Once full White (> 40 km/h), Cyan color fills up from bottom over the White bar
        if p_cyan > 0.001 {
            let left_c = Self::sub_path(l_bottom, l_ctrl_low, l_ctrl_high, l_top, p_cyan);
            let right_c = Self::sub_path(r_bottom, r_ctrl_low, r_ctrl_high, r_top, p_cyan);
            Self::stroke_path(pixmap, &left_c, cyan_glow, glow_width, BlendMode::Screen);
            Self::stroke_path(pixmap, &right_c, cyan_glow, glow_width, BlendMode::Screen);
            Self::stroke_path(pixmap, &left_c, cyan_core, core_width, BlendMode::SourceOver);
            Self::stroke_path(pixmap, &right_c, cyan_core, core_width, BlendMode::SourceOver);
        }

        // 3) Once full Cyan (> 70 km/h), Yellow fills up from bottom over the Cyan bar
        if p_yellow > 0.001 {
            let left_y = Self::sub_path(l_bottom, l_ctrl_low, l_ctrl_high, l_top, p_yellow);
            let right_y = Self::sub_path(r_bottom, r_ctrl_low, r_ctrl_high, r_top, p_yellow);
            Self::stroke_path(pixmap, &left_y, yellow_glow, glow_width, BlendMode::Screen);
            Self::stroke_path(pixmap, &right_y, yellow_glow, glow_width, BlendMode::Screen);
            Self::stroke_path(pixmap, &left_y, yellow_core, core_width, BlendMode::SourceOver);
            Self::stroke_path(pixmap, &right_y, yellow_core, core_width, BlendMode::SourceOver);
        }

        // 4) Once full Yellow (> 95 km/h), Red fills up from bottom over the Yellow bar
        if p_red > 0.001 {
            let left_r = Self::sub_path(l_bottom, l_ctrl_low, l_ctrl_high, l_top, p_red);
            let right_r = Self::sub_path(r_bottom, r_ctrl_low, r_ctrl_high, r_top, p_red);
            Self::stroke_path(pixmap, &left_r, red_glow, glow_width * 1.1, BlendMode::Screen);
            Self::stroke_path(pixmap, &right_r, red_glow, glow_width * 1.1, BlendMode::Screen);
            Self::stroke_path(pixmap, &left_r, red_core, core_width, BlendMode::SourceOver);
            Self::stroke_path(pixmap, &right_r, red_core, core_width, BlendMode::SourceOver);
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
