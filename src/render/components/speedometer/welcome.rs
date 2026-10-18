// Startup Welcome & Greeting animation: Emo companion sleeping and waking up!
// Sequence:
// 1. 0.0s - 1.5s: Sleeping peacefully with floating "Z z z" and gauge brackets check.
// 2. 1.5s - 2.8s: Waking up, yawning widely, blinking eyes open with an awakening energy ripple.
// 3. 2.8s - 4.2s: Fully awake, beaming a happy smile with "WELCOME" & "● READY TO DRIVE".
// 4. 4.2s+: Seamless handoff to autonomous Emo companion mode (or Speedometer if driving).

use crate::render::components::emo::{EmoExpression, EmoSpriteRenderer};
use crate::render::core::FontRenderer;
use tiny_skia::{
    BlendMode, Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform,
};

pub struct SpeedometerWelcome;

impl SpeedometerWelcome {
    pub fn draw(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        cx: f32,
        cy: f32,
        time: f32,
        alpha: f32,
    ) {
        if alpha <= 0.001 {
            return;
        }

        let emo_cy = cy - 18.0;

        if time < 1.5 {
            // ==========================================
            // PHASE 1: SLEEPING (0.0s - 1.5s)
            // ==========================================
            // Emo is sleeping with closed eyes (sleepy.svg, index 26)
            let mood = EmoExpression::Atlas(26);
            EmoSpriteRenderer::draw(pixmap, cx, emo_cy, time, alpha, mood);

            // Floating animated "Z z z" drifting upward to the right
            Self::draw_floating_zzz(pixmap, fonts, cx, emo_cy, time, alpha);

            // Subtitle below
            let standby_text = "STANDBY";
            let sz = 13.0;
            let w = fonts.measure_width(&fonts.regular, standby_text, sz);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                standby_text,
                cx - (w / 2.0),
                emo_cy + 82.0,
                sz,
                Color::from_rgba8(140, 150, 165, (160.0 * alpha) as u8),
            );
        } else if time < 2.8 {
            // ==========================================
            // PHASE 2: WAKING UP & YAWNING (1.5s - 2.8s)
            // ==========================================
            let wake_time = time - 1.5;
            // Emo yawns and blinks eyes open (ngap.svg, index 18)
            let mood = EmoExpression::Atlas(18);
            EmoSpriteRenderer::draw(pixmap, cx, emo_cy, wake_time, alpha, mood);

            // Awakening energy ripple ring expanding outward from center
            Self::draw_awakening_ripple(pixmap, cx, emo_cy, wake_time, alpha);

            // Subtitle: WAKING UP...
            let wake_text = "WAKING UP...";
            let sz = 15.0;
            let w = fonts.measure_width(&fonts.bold, wake_text, sz);
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                wake_text,
                cx - (w / 2.0),
                emo_cy + 82.0,
                sz,
                Color::from_rgba8(0, 230, 255, (230.0 * alpha) as u8),
            );

            // Micro progress scanner line underneath
            Self::draw_mini_scanner(pixmap, cx, emo_cy + 92.0, wake_time / 1.3, alpha);
        } else {
            // ==========================================
            // PHASE 3: AWAKE, HAPPY & GREETING (2.8s - 4.2s)
            // ==========================================
            let greet_time = time - 2.8;
            // Emo is wide awake beaming a bright happy smile (cuoi.svg, index 5)
            let mood = EmoExpression::Atlas(5);
            EmoSpriteRenderer::draw(pixmap, cx, emo_cy, greet_time, alpha, mood);

            // Greeting Headline: "W E L C O M E"
            let welcome_text = "W E L C O M E";
            let sz_w = 26.0;
            let w = fonts.measure_width(&fonts.bold, welcome_text, sz_w);
            let tx = cx - (w / 2.0);
            let ty = emo_cy + 78.0;

            // Soft cyan glow backdrop
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                welcome_text,
                tx,
                ty + 1.0,
                sz_w,
                Color::from_rgba8(0, 210, 255, (50.0 * alpha) as u8),
            );
            // Crisp pure white main text
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                welcome_text,
                tx,
                ty,
                sz_w,
                Color::from_rgba8(255, 255, 255, (255.0 * alpha) as u8),
            );

            // Status Badge: ● READY TO DRIVE
            let status_text = "● READY TO DRIVE";
            let sz_s = 13.5;
            let sw = fonts.measure_width(&fonts.bold, status_text, sz_s);
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                status_text,
                cx - (sw / 2.0),
                emo_cy + 100.0,
                sz_s,
                Color::from_rgba8(78, 236, 213, (240.0 * alpha) as u8),
            );
        }
    }

    /// Floating "Z z z" animation drifting upward to the top-right
    fn draw_floating_zzz(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        cx: f32,
        cy: f32,
        time: f32,
        alpha: f32,
    ) {
        let z_items = [
            ("Z", 24.0, 70.0, 40.0, 0.0),
            ("z", 18.0, 92.0, 62.0, 0.35),
            ("z", 13.0, 110.0, 84.0, 0.7),
        ];

        for (txt, sz, off_x, base_y, phase) in z_items {
            let cycle = ((time + phase) * 18.0) % 38.0;
            let float_y = base_y + cycle;
            let sway = (time * 3.0 + phase * 4.0).sin() * 4.5;
            let fade = (1.0 - (cycle / 38.0)).clamp(0.0, 1.0) * alpha;

            let color = Color::from_rgba8(160, 225, 255, (fade * 210.0) as u8);
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                txt,
                cx + off_x + sway,
                cy - float_y,
                sz,
                color,
            );
        }
    }

    /// Awakening pulse ripple ring that expands outward from center
    fn draw_awakening_ripple(pixmap: &mut Pixmap, cx: f32, cy: f32, wake_time: f32, alpha: f32) {
        if wake_time > 1.0 {
            return;
        }
        let p = (wake_time / 0.9).clamp(0.0, 1.0);
        let r = 40.0 + 110.0 * p;
        let a = ((1.0 - p) * 0.7 * alpha).clamp(0.0, 1.0);

        if a <= 0.001 || r <= 1.0 {
            return;
        }

        let mut pb = PathBuilder::new();
        let steps = 40;
        for i in 0..=steps {
            let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            if i == 0 {
                pb.move_to(x, y);
            } else {
                pb.line_to(x, y);
            }
        }
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(0, 235, 255, (a * 255.0) as u8);
            paint.anti_alias = true;
            paint.blend_mode = BlendMode::SourceOver;
            let stroke = Stroke {
                width: 2.0,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    /// Horizontal scanner line under waking up status
    fn draw_mini_scanner(pixmap: &mut Pixmap, cx: f32, y: f32, progress: f32, alpha: f32) {
        let half_w = 60.0;
        let mut pb_bg = PathBuilder::new();
        pb_bg.move_to(cx - half_w, y);
        pb_bg.line_to(cx + half_w, y);
        if let Some(path) = pb_bg.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(30, 80, 110, (120.0 * alpha) as u8);
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.5,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }

        // Active sweep bar
        let fill_w = half_w * 2.0 * progress.clamp(0.0, 1.0);
        let mut pb_fill = PathBuilder::new();
        pb_fill.move_to(cx - half_w, y);
        pb_fill.line_to(cx - half_w + fill_w, y);
        if let Some(path) = pb_fill.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(0, 240, 255, (230.0 * alpha) as u8);
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 2.0,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}
