// Dynamic speed telemetry waveform chart panel rendered with tiny-skia.
// Features a single smooth red Catmull-Rom curve, subtle vertical
// gradient area fill, a single glowing endpoint dot at the far right,
// and a compact GPS status indicator (icon + "GPS") on the bottom right.

use crate::models::vehicle::state::VehicleState;
use crate::render::core::FontRenderer;
use std::collections::VecDeque;
use tiny_skia::{
    BlendMode, Color, FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Paint, Path,
    PathBuilder, Pixmap, Point, SpreadMode, Stroke, Transform,
};

const CHART_LEFT: f32 = 35.0;
const CHART_RIGHT: f32 = 390.0;
const CHART_WIDTH: f32 = CHART_RIGHT - CHART_LEFT; // 355.0
const CHART_TOP: f32 = 75.0;
const BASELINE_Y: f32 = 365.0;
const NUM_SAMPLE_POINTS: usize = 9;

pub struct ChartPanel {
    samples: VecDeque<f32>,
    phase: f32,
    sample_timer: f32,
}

impl ChartPanel {
    pub fn new() -> Self {
        let mut samples = VecDeque::with_capacity(NUM_SAMPLE_POINTS);
        // Initial undulating wave samples
        let initial_vals = [25.0, 10.0, 38.0, 42.0, 52.0, 30.0, 75.0, 50.0, 58.0];
        for &val in &initial_vals {
            samples.push_back(val);
        }

        Self {
            samples,
            phase: 0.0,
            sample_timer: 0.0,
        }
    }

    pub fn draw(&mut self, pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState, dt: f32) {
        self.update(state.current_speed, dt);

        // 1. Compute control points from samples for the active speed wave
        let coords = self.compute_screen_coords();

        // 2. Build smooth Catmull-Rom spline path
        if let Some((stroke_path, area_path)) = build_spline_paths(&coords) {
            // Draw red gradient area under the curve
            let mut area_paint = Paint::default();
            area_paint.anti_alias = true;
            if let Some(shader) = LinearGradient::new(
                Point::from_xy(0.0, CHART_TOP),
                Point::from_xy(0.0, BASELINE_Y),
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(244, 63, 94, 125)),
                    GradientStop::new(0.45, Color::from_rgba8(225, 29, 72, 45)),
                    GradientStop::new(0.85, Color::from_rgba8(159, 18, 57, 10)),
                    GradientStop::new(1.0, Color::from_rgba8(76, 5, 25, 0)),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                area_paint.shader = shader;
                pixmap.fill_path(
                    &area_path,
                    &area_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }

            // Draw soft outer red glow
            let mut glow_paint = Paint::default();
            glow_paint.set_color(Color::from_rgba8(244, 63, 94, 65));
            glow_paint.anti_alias = true;
            glow_paint.blend_mode = BlendMode::Screen;
            let glow_stroke = Stroke {
                width: 8.0,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&stroke_path, &glow_paint, &glow_stroke, Transform::identity(), None);

            // Draw solid core vibrant red stroke
            let mut core_paint = Paint::default();
            core_paint.set_color(Color::from_rgba8(255, 45, 75, 255));
            core_paint.anti_alias = true;
            let core_stroke = Stroke {
                width: 3.5,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&stroke_path, &core_paint, &core_stroke, Transform::identity(), None);
        }

        // 3. Draw ONLY the single glowing red endpoint dot at the far right
        if let Some(&(last_x, last_y)) = coords.last() {
            draw_circle(pixmap, last_x, last_y, 9.0, Color::from_rgba8(255, 45, 75, 75), true);
            draw_circle(pixmap, last_x, last_y, 5.0, Color::from_rgba8(255, 70, 95, 255), true);
        }

        // 4. GPS Status Indicator directly below chart on the right:
        // Clear bright white if connected, dimmed translucent if not connected.
        // Displays only the GPS icon and the text "GPS".
        let gps_color = if state.gps_connected {
            Color::from_rgba8(255, 255, 255, 255) // Màu trắng rõ nếu đã connect
        } else {
            Color::from_rgba8(95, 100, 110, 255) // Hiển thị mờ nếu chưa connect
        };

        let font_size = 11.0;
        let text_w = fonts.measure_width(&fonts.bold, "GPS", font_size);
        let text_x = CHART_RIGHT - text_w;
        let text_y = BASELINE_Y + 8.0; // 377.0
        let icon_cx = text_x - 11.0;
        let icon_cy = text_y - 2.0; // 373.0

        draw_gps_icon(pixmap, icon_cx, icon_cy, gps_color);
        fonts.draw_text(
            pixmap,
            &fonts.bold,
            "GPS",
            text_x,
            text_y,
            font_size,
            gps_color,
        );
    }

    fn update(&mut self, speed: f32, dt: f32) {
        self.phase = (self.phase + dt * 1.5).rem_euclid(std::f32::consts::TAU * 4.0);
        self.sample_timer += dt;

        // Sample speed at periodic intervals (~0.25s) to scroll waveform smoothly
        if self.sample_timer >= 0.25 {
            self.sample_timer -= 0.25;
            self.samples.pop_front();
            // Wave amplitude dynamically scales with vehicle movement
            let base_speed = speed.max(10.0);
            let wave_amp = (speed * 0.35 + 8.0).min(30.0);
            let ripple = (self.phase * 2.1).sin() * wave_amp + (self.phase * 3.6).cos() * (wave_amp * 0.35);
            let target_sample = (base_speed + ripple).max(5.0);
            self.samples.push_back(target_sample);
        } else if let Some(last) = self.samples.back_mut() {
            let base_speed = speed.max(10.0);
            let wave_amp = (speed * 0.35 + 8.0).min(30.0);
            let ripple = (self.phase * 2.1).sin() * wave_amp + (self.phase * 3.6).cos() * (wave_amp * 0.35);
            let target = (base_speed + ripple).max(5.0);
            *last += (target - *last) * (dt * 8.0).min(1.0);
        }
    }

    fn compute_screen_coords(&self) -> Vec<(f32, f32)> {
        let n = self.samples.len().max(2);
        let max_val = self
            .samples
            .iter()
            .cloned()
            .fold(50.0f32, |a, b| a.max(b))
            .max(50.0);

        self.samples
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                let x = CHART_LEFT + (i as f32 / (n - 1) as f32) * CHART_WIDTH;
                let ratio = (val / (max_val * 1.18)).clamp(0.08, 0.92);
                let y = BASELINE_Y - ratio * (BASELINE_Y - CHART_TOP);
                (x, y)
            })
            .collect()
    }
}

/// Builds Catmull-Rom cubic Bezier spline for both the stroke and the closed area
fn build_spline_paths(pts: &[(f32, f32)]) -> Option<(Path, Path)> {
    if pts.len() < 2 {
        return None;
    }

    let mut stroke_pb = PathBuilder::new();
    let mut area_pb = PathBuilder::new();

    stroke_pb.move_to(pts[0].0, pts[0].1);
    area_pb.move_to(pts[0].0, pts[0].1);

    for i in 0..pts.len() - 1 {
        let p0 = if i == 0 { pts[0] } else { pts[i - 1] };
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = if i + 2 < pts.len() { pts[i + 2] } else { p2 };

        // Standard Catmull-Rom to cubic Bezier conversion
        let c1_x = p1.0 + (p2.0 - p0.0) / 6.0;
        let c1_y = p1.1 + (p2.1 - p0.1) / 6.0;
        let c2_x = p2.0 - (p3.0 - p1.0) / 6.0;
        let c2_y = p2.1 - (p3.1 - p1.1) / 6.0;

        stroke_pb.cubic_to(c1_x, c1_y, c2_x, c2_y, p2.0, p2.1);
        area_pb.cubic_to(c1_x, c1_y, c2_x, c2_y, p2.0, p2.1);
    }

    let last_pt = pts[pts.len() - 1];
    area_pb.line_to(last_pt.0, BASELINE_Y);
    area_pb.line_to(pts[0].0, BASELINE_Y);
    area_pb.close();

    Some((stroke_pb.finish()?, area_pb.finish()?))
}

fn draw_circle(pixmap: &mut Pixmap, cx: f32, cy: f32, r: f32, color: Color, filled: bool) {
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, r);
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        if filled {
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
        } else {
            let stroke = Stroke {
                width: 1.5,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

fn draw_gps_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
    let mut pb = PathBuilder::new();
    pb.move_to(cx, cy - 6.0);
    pb.line_to(cx + 4.5, cy + 5.0);
    pb.line_to(cx, cy + 2.5);
    pb.line_to(cx - 4.5, cy + 5.0);
    pb.close();

    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }
}
