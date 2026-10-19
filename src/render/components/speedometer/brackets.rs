// Premium speedometer brackets with a restrained track, smooth reveal and a
// single continuous speed gradient shared with the digits.

use super::tier::get_tier_colors;
use tiny_skia::{
    BlendMode, Color, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke,
    Transform,
};

pub struct SpeedometerBrackets;

impl SpeedometerBrackets {
    pub fn draw(pixmap: &mut Pixmap, speed: f32, fly_progress: f32, alpha: f32) {
        if alpha <= 0.001 || fly_progress <= 0.001 {
            return;
        }

        let cx = 640.0;
        let fly = ease_out_cubic(fly_progress.clamp(0.0, 1.0));
        let left = Curve::new(
            (cx + (530.0 - cx) * fly, 348.0),
            (cx + (450.0 - cx) * fly, 305.0),
            (cx + (450.0 - cx) * fly, 175.0),
            (cx + (530.0 - cx) * fly, 132.0),
        );
        let right = Curve::new(
            (cx + (750.0 - cx) * fly, 348.0),
            (cx + (830.0 - cx) * fly, 305.0),
            (cx + (830.0 - cx) * fly, 175.0),
            (cx + (750.0 - cx) * fly, 132.0),
        );

        let scale = 0.65 + 0.35 * fly;
        let track = Color::from_rgba8(42, 50, 58, opacity(0.62, alpha));
        let track_glow = Color::from_rgba8(70, 82, 92, opacity(0.10, alpha));

        // A quiet track keeps the bracket readable without looking like a
        // thick neon tube when the car is stationary.
        for curve in [&left, &right] {
            stroke_path(
                pixmap,
                &curve.full_path(),
                track_glow,
                15.0 * scale,
                BlendMode::Screen,
            );
            stroke_path(
                pixmap,
                &curve.full_path(),
                track,
                5.0 * scale,
                BlendMode::SourceOver,
            );
        }

        let progress = ease_in_out((speed / 100.0).clamp(0.0, 1.0));
        if progress <= 0.001 {
            return;
        }

        let (core, glow) = get_tier_colors(speed);
        let active_core = with_alpha(core, alpha * 0.98);
        let active_glow = with_alpha(glow, alpha * 0.36);
        let active_soft = with_alpha(glow, alpha * 0.16);

        for curve in [&left, &right] {
            let active_path = curve.sub_path(progress);
            stroke_path(
                pixmap,
                &active_path,
                active_soft,
                24.0 * scale,
                BlendMode::Screen,
            );
            stroke_path(
                pixmap,
                &active_path,
                active_glow,
                12.0 * scale,
                BlendMode::Screen,
            );
            stroke_path(
                pixmap,
                &active_path,
                active_core,
                6.5 * scale,
                BlendMode::SourceOver,
            );

            // A small moving cap makes the fill feel continuous at 60 FPS.
            let (x, y) = curve.point_at(progress);
            draw_dot(pixmap, x, y, 7.0 * scale, active_glow, BlendMode::Screen);
            draw_dot(
                pixmap,
                x,
                y,
                2.8 * scale,
                active_core,
                BlendMode::SourceOver,
            );
        }
    }
}

struct Curve {
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
}

impl Curve {
    fn new(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> Self {
        Self { p0, p1, p2, p3 }
    }

    fn full_path(&self) -> Option<Path> {
        self.sub_path(1.0)
    }

    fn sub_path(&self, progress: f32) -> Option<Path> {
        let p = progress.clamp(0.0, 1.0);
        if p <= 0.001 {
            return None;
        }
        let lerp = |a: (f32, f32), b: (f32, f32), amount: f32| {
            (a.0 + (b.0 - a.0) * amount, a.1 + (b.1 - a.1) * amount)
        };
        let (q0, q1, q2, q3) = if p >= 0.999 {
            (self.p0, self.p1, self.p2, self.p3)
        } else {
            let q1 = lerp(self.p0, self.p1, p);
            let r0 = lerp(self.p1, self.p2, p);
            let r1 = lerp(self.p2, self.p3, p);
            let q2 = lerp(q1, r0, p);
            let s0 = lerp(r0, r1, p);
            let q3 = lerp(q2, s0, p);
            (self.p0, q1, q2, q3)
        };

        let mut builder = PathBuilder::new();
        builder.move_to(q0.0, q0.1);
        builder.cubic_to(q1.0, q1.1, q2.0, q2.1, q3.0, q3.1);
        builder.finish()
    }

    fn point_at(&self, progress: f32) -> (f32, f32) {
        let t = progress.clamp(0.0, 1.0);
        let one_minus_t = 1.0 - t;
        let a = one_minus_t.powi(3);
        let b = 3.0 * one_minus_t.powi(2) * t;
        let c = 3.0 * one_minus_t * t.powi(2);
        let d = t.powi(3);
        (
            a * self.p0.0 + b * self.p1.0 + c * self.p2.0 + d * self.p3.0,
            a * self.p0.1 + b * self.p1.1 + c * self.p2.1 + d * self.p3.1,
        )
    }
}

fn stroke_path(
    pixmap: &mut Pixmap,
    path: &Option<Path>,
    color: Color,
    width: f32,
    blend_mode: BlendMode,
) {
    let Some(path) = path else {
        return;
    };
    if color.alpha() <= 0.001 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    paint.blend_mode = blend_mode;
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    };
    pixmap.stroke_path(path, &paint, &stroke, Transform::identity(), None);
}

fn draw_dot(pixmap: &mut Pixmap, x: f32, y: f32, radius: f32, color: Color, blend_mode: BlendMode) {
    if color.alpha() <= 0.001 {
        return;
    }
    let mut path_builder = PathBuilder::new();
    path_builder.push_circle(x, y, radius);
    let Some(path) = path_builder.finish() else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    paint.blend_mode = blend_mode;
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

fn with_alpha(color: Color, opacity: f32) -> Color {
    Color::from_rgba8(
        (color.red() * 255.0).round() as u8,
        (color.green() * 255.0).round() as u8,
        (color.blue() * 255.0).round() as u8,
        (color.alpha() * opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn opacity(base: f32, alpha: f32) -> u8 {
    (base * alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn ease_in_out(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn ease_out_cubic(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}
