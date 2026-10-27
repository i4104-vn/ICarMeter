// Real-time speed waveform chart panel rendered with tiny-skia.
// Features a smooth Catmull-Rom spline curve tracking actual vehicle speed,
// continuous sub-pixel scrolling, dynamic scale easing, pulsing endpoint aura,
// subtle baseline axis, and GPS status indicator.

use crate::models::vehicle::state::VehicleState;
use crate::render::core::FontRenderer;
use std::collections::VecDeque;
use tiny_skia::{
    BlendMode, Color, FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Paint, Path,
    PathBuilder, Pixmap, Point, Rect, SpreadMode, Stroke, Transform,
};

const CHART_LEFT: f32 = 35.0;
const CHART_RIGHT: f32 = 390.0;
const CHART_WIDTH: f32 = CHART_RIGHT - CHART_LEFT; // 355.0
const BASELINE_Y: f32 = 350.0;
const CHART_HEIGHT: f32 = 140.0; // Reduced height to keep comfortable headroom
const CHART_TOP: f32 = BASELINE_Y - CHART_HEIGHT; // 210.0
const NUM_SAMPLE_POINTS: usize = 72;
const SAMPLE_INTERVAL: f32 = 0.1; // 100ms per sample (7.2s window)
const BASE_MAX_SPEED: f32 = 140.0; // Fixed automotive scale ceiling (with headroom if exceeding)

pub struct ChartPanel {
    samples: VecDeque<f32>,
    visual_speed: f32,
    max_scale: f32,
    sample_timer: f32,
    pulse_time: f32,
}

impl ChartPanel {
    pub fn new() -> Self {
        let mut samples = VecDeque::with_capacity(NUM_SAMPLE_POINTS);
        for _ in 0..NUM_SAMPLE_POINTS {
            samples.push_back(0.0);
        }

        Self {
            samples,
            visual_speed: 0.0,
            max_scale: BASE_MAX_SPEED,
            sample_timer: 0.0,
            pulse_time: 0.0,
        }
    }

    pub fn update(&mut self, speed: f32, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        self.pulse_time = (self.pulse_time + dt).rem_euclid(1000.0);

        // Smoothly filter incoming vehicle speed to eliminate sensor jitter while remaining responsive
        let target_speed = speed.max(0.0);
        let speed_response = 1.0 - (-dt * 12.0).exp();
        self.visual_speed += (target_speed - self.visual_speed) * speed_response;
        if target_speed <= 0.01 && self.visual_speed < 0.05 {
            self.visual_speed = 0.0;
        }

        // Fixed automotive baseline scale (140 km/h) with 25% headroom if vehicle exceeds 112 km/h
        let current_peak = self
            .samples
            .iter()
            .copied()
            .fold(self.visual_speed, |a, b| a.max(b));
        let target_max = (current_peak * 1.25).max(BASE_MAX_SPEED);
        let scale_rate = if target_max > self.max_scale { 3.5 } else { 1.2 };
        self.max_scale += (target_max - self.max_scale) * (dt * scale_rate).min(1.0);

        // Continuous time-based sampling
        self.sample_timer += dt;
        while self.sample_timer >= SAMPLE_INTERVAL {
            self.sample_timer -= SAMPLE_INTERVAL;
            self.samples.pop_front();
            self.samples.push_back(self.visual_speed);
        }
    }

    pub fn draw(
        &mut self,
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        state: &VehicleState,
        dt: f32,
    ) {
        self.update(state.current_speed, dt);

        // 1. Compute screen coordinates
        let coords = self.compute_screen_coords();

        // 2. Build smooth Catmull-Rom spline path
        if let Some((stroke_path, area_path)) = build_spline_paths(&coords) {
            // Draw gradient area under the curve
            let mut area_paint = Paint::default();
            area_paint.anti_alias = true;
            if let Some(shader) = LinearGradient::new(
                Point::from_xy(0.0, CHART_TOP),
                Point::from_xy(0.0, BASELINE_Y),
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(244, 63, 94, 120)),
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

            // Draw outer red glow
            let mut glow_paint = Paint::default();
            glow_paint.set_color(Color::from_rgba8(244, 63, 94, 55));
            glow_paint.anti_alias = true;
            glow_paint.blend_mode = BlendMode::Screen;
            let glow_stroke = Stroke {
                width: 7.0,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(
                &stroke_path,
                &glow_paint,
                &glow_stroke,
                Transform::identity(),
                None,
            );

            // Draw crisp core vibrant red stroke
            let mut core_paint = Paint::default();
            core_paint.set_color(Color::from_rgba8(255, 45, 80, 255));
            core_paint.anti_alias = true;
            let core_stroke = Stroke {
                width: 2.8,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(
                &stroke_path,
                &core_paint,
                &core_stroke,
                Transform::identity(),
                None,
            );
        }

        // 3. Left edge soft fade to black so the historical tail dissolves seamlessly
        let fade_w = 40.0;
        if let Some(rect) = Rect::from_xywh(
            CHART_LEFT - 10.0,
            CHART_TOP - 10.0,
            fade_w + 10.0,
            CHART_HEIGHT + 20.0,
        ) {
            let mut fade_paint = Paint::default();
            if let Some(shader) = LinearGradient::new(
                Point::from_xy(CHART_LEFT - 2.0, 0.0),
                Point::from_xy(CHART_LEFT + fade_w, 0.0),
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(0, 0, 0, 255)),
                    GradientStop::new(0.35, Color::from_rgba8(0, 0, 0, 180)),
                    GradientStop::new(0.7, Color::from_rgba8(0, 0, 0, 70)),
                    GradientStop::new(1.0, Color::from_rgba8(0, 0, 0, 0)),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                fade_paint.shader = shader;
                fade_paint.anti_alias = true;
                pixmap.fill_rect(rect, &fade_paint, Transform::identity(), None);
            }
        }

        // 4. Draw subtle ground baseline axis
        let mut base_pb = PathBuilder::new();
        base_pb.move_to(CHART_LEFT, BASELINE_Y);
        base_pb.line_to(CHART_RIGHT, BASELINE_Y);
        if let Some(path) = base_pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(55, 65, 81, 120));
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.0,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }

        // 5. Draw glowing animated endpoint dot and vertical telemetry drop-line
        if let Some(&(last_x, last_y)) = coords.last() {
            // Vertical indicator drop-line down to baseline
            if last_y < BASELINE_Y - 2.0 {
                let mut pb = PathBuilder::new();
                pb.move_to(last_x, last_y + 3.0);
                pb.line_to(last_x, BASELINE_Y);
                if let Some(path) = pb.finish() {
                    let mut line_paint = Paint::default();
                    line_paint.set_color(Color::from_rgba8(255, 65, 95, 55));
                    line_paint.anti_alias = true;
                    let stroke = Stroke {
                        width: 1.2,
                        ..Default::default()
                    };
                    pixmap.stroke_path(&path, &line_paint, &stroke, Transform::identity(), None);
                }
            }

            // Pulsing halo aura
            let pulse = (self.pulse_time * 4.0).sin() * 0.5 + 0.5;
            let speed_factor = (self.visual_speed / 50.0).clamp(0.0, 1.0);
            let halo_r = 7.0 + pulse * 3.5 + speed_factor * 2.0;
            let halo_alpha = (40.0 + pulse * 35.0 + speed_factor * 30.0) as u8;

            draw_circle(
                pixmap,
                last_x,
                last_y,
                halo_r,
                Color::from_rgba8(255, 45, 80, halo_alpha),
                true,
                BlendMode::Screen,
            );
            draw_circle(
                pixmap,
                last_x,
                last_y,
                4.5,
                Color::from_rgba8(255, 75, 105, 200),
                true,
                BlendMode::SourceOver,
            );
            draw_circle(
                pixmap,
                last_x,
                last_y,
                2.6,
                Color::from_rgba8(255, 240, 245, 255),
                true,
                BlendMode::SourceOver,
            );
        }

        // 5. GPS Status Indicator directly below chart on the right:
        let gps_color = if state.gps_connected {
            Color::from_rgba8(255, 255, 255, 255)
        } else {
            Color::from_rgba8(95, 100, 110, 255)
        };

        let font_size = 11.0;
        let text_w = fonts.measure_width(&fonts.bold, "GPS", font_size);
        let text_x = CHART_RIGHT - text_w;
        let text_y = BASELINE_Y + 12.0;
        let icon_cx = text_x - 11.0;
        let icon_cy = text_y - 2.0;

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

    fn compute_screen_coords(&self) -> Vec<(f32, f32)> {
        let n = self.samples.len().max(2);
        let step_x = CHART_WIDTH / (n - 1) as f32;
        let sub_offset = (self.sample_timer / SAMPLE_INTERVAL).clamp(0.0, 1.0);
        let scale = self.max_scale.max(10.0);

        let mut raw_pts = Vec::with_capacity(n + 1);
        for (i, &val) in self.samples.iter().enumerate() {
            let x = CHART_LEFT + (i as f32 - sub_offset) * step_x;
            let ratio = (val / scale).clamp(0.0, 1.0);
            let y = BASELINE_Y - ratio * CHART_HEIGHT;
            raw_pts.push((x, y));
        }

        // Live endpoint currently at CHART_RIGHT
        let live_ratio = (self.visual_speed / scale).clamp(0.0, 1.0);
        let live_y = BASELINE_Y - live_ratio * CHART_HEIGHT;
        raw_pts.push((CHART_RIGHT, live_y));

        // Ensure the left-most point starts cleanly at CHART_LEFT
        if raw_pts[0].0 < CHART_LEFT && raw_pts.len() > 1 {
            let (x0, y0) = raw_pts[0];
            let (x1, y1) = raw_pts[1];
            let t = if (x1 - x0).abs() > 0.001 {
                ((CHART_LEFT - x0) / (x1 - x0)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let y_at_left = y0 + t * (y1 - y0);
            raw_pts[0] = (CHART_LEFT, y_at_left);
        }

        raw_pts
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

        // Standard Catmull-Rom to cubic Bezier conversion with bounded clamp
        let c1_x = (p1.0 + (p2.0 - p0.0) / 6.0).clamp(CHART_LEFT, CHART_RIGHT);
        let c1_y = (p1.1 + (p2.1 - p0.1) / 6.0).clamp(CHART_TOP, BASELINE_Y);
        let c2_x = (p2.0 - (p3.0 - p1.0) / 6.0).clamp(CHART_LEFT, CHART_RIGHT);
        let c2_y = (p2.1 - (p3.1 - p1.1) / 6.0).clamp(CHART_TOP, BASELINE_Y);

        stroke_pb.cubic_to(c1_x, c1_y, c2_x, c2_y, p2.0, p2.1);
        area_pb.cubic_to(c1_x, c1_y, c2_x, c2_y, p2.0, p2.1);
    }

    let last_pt = pts[pts.len() - 1];
    area_pb.line_to(last_pt.0, BASELINE_Y);
    area_pb.line_to(pts[0].0, BASELINE_Y);
    area_pb.close();

    Some((stroke_pb.finish()?, area_pb.finish()?))
}

fn draw_circle(
    pixmap: &mut Pixmap,
    cx: f32,
    cy: f32,
    r: f32,
    color: Color,
    filled: bool,
    blend_mode: BlendMode,
) {
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, r);
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        paint.blend_mode = blend_mode;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_panel_initial_state() {
        let chart = ChartPanel::new();
        assert_eq!(chart.samples.len(), NUM_SAMPLE_POINTS);
        assert_eq!(chart.visual_speed, 0.0);
        for &s in &chart.samples {
            assert_eq!(s, 0.0);
        }

        let coords = chart.compute_screen_coords();
        assert!(!coords.is_empty());
        for &(x, y) in &coords {
            assert!(x >= CHART_LEFT - 0.01 && x <= CHART_RIGHT + 0.01);
            assert_eq!(y, BASELINE_Y);
        }
    }

    #[test]
    fn test_chart_panel_tracks_speed() {
        let mut chart = ChartPanel::new();
        // Simulate vehicle accelerating to 40 km/h over 1 second
        for _ in 0..60 {
            chart.update(40.0, 1.0 / 60.0);
        }

        assert!(chart.visual_speed > 35.0, "Visual speed should track actual speed");
        assert!(chart.samples.back().copied().unwrap() > 30.0);

        let coords = chart.compute_screen_coords();
        for &(x, y) in &coords {
            assert!(!x.is_nan());
            assert!(!y.is_nan());
            assert!(x >= CHART_LEFT - 0.01 && x <= CHART_RIGHT + 0.01);
            assert!(y >= CHART_TOP && y <= BASELINE_Y);
        }

        let paths = build_spline_paths(&coords);
        assert!(paths.is_some());
    }

    #[test]
    fn test_chart_panel_brakes_to_zero() {
        let mut chart = ChartPanel::new();
        // First accelerate
        for _ in 0..60 {
            chart.update(50.0, 1.0 / 60.0);
        }
        // Then brake to 0 and wait 8 seconds
        for _ in 0..480 {
            chart.update(0.0, 1.0 / 60.0);
        }

        assert_eq!(chart.visual_speed, 0.0);
        for &s in &chart.samples {
            assert_eq!(s, 0.0);
        }
    }
}

