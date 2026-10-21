// MapLibre Navigation Vehicle Cursor Arrow
// In Heading-Up navigation mode, the arrow is permanently anchored facing straight UP (0°).
// Includes tactical pulsing radar ring, drop-shadow halo, and sharp arrow geometry.

use super::style::MapLibreStyle;
use tiny_skia::{Color, FillRule, LineJoin, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

pub struct VehicleArrow;

impl VehicleArrow {
    /// Draws the vehicle navigation cursor at (cx, cy) pointing strictly UP
    pub fn draw(pixmap: &mut Pixmap, style: &MapLibreStyle, cx: f32, cy: f32, pulse_phase: f32) {
        // 1. Radar Pulse Ring (Concentric acoustic wave expanding outwards)
        let pulse_r = 18.0 + pulse_phase.sin().abs() * 14.0;
        let pulse_alpha = (1.0 - (pulse_r - 18.0) / 14.0).clamp(0.0, 1.0) * 120.0;
        let mut pb_pulse = PathBuilder::new();
        pb_pulse.push_circle(cx, cy, pulse_r);
        if let Some(path) = pb_pulse.finish() {
            let mut pulse_paint = Paint::default();
            pulse_paint.set_color(Color::from_rgba8(56, 189, 248, pulse_alpha as u8));
            let stroke = Stroke {
                width: 1.5,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &pulse_paint, &stroke, Transform::identity(), None);
        }

        // 2. Drop Shadow & Protective Halo Ring
        let mut pb_ring = PathBuilder::new();
        pb_ring.push_circle(cx, cy, 14.0);
        if let Some(path) = pb_ring.finish() {
            let mut ring_paint = Paint::default();
            ring_paint.set_color(style.arrow_halo);
            pixmap.fill_path(&path, &ring_paint, FillRule::Winding, Transform::identity(), None);
        }

        // 3. Navigation Arrow Polygon (Strictly pointing straight UP: 0° relative to screen)
        let mut pb_arrow = PathBuilder::new();
        pb_arrow.move_to(cx, cy - 16.0);          // Tip pointing North
        pb_arrow.line_to(cx + 10.0, cy + 10.0);   // Right wing
        pb_arrow.line_to(cx, cy + 5.0);           // Inner cockpit notch
        pb_arrow.line_to(cx - 10.0, cy + 10.0);   // Left wing
        pb_arrow.close();

        if let Some(path) = pb_arrow.finish() {
            let mut arrow_paint = Paint::default();
            arrow_paint.set_color(style.arrow_color);
            pixmap.fill_path(&path, &arrow_paint, FillRule::Winding, Transform::identity(), None);

            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(style.arrow_stroke);
            let stroke = Stroke {
                width: 1.4,
                line_join: LineJoin::Miter,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }

        // 4. Center Cockpit Dot
        if let Some(dot) = Rect::from_xywh(cx - 1.5, cy - 3.0, 3.0, 3.0) {
            let mut dot_paint = Paint::default();
            dot_paint.set_color(Color::WHITE);
            pixmap.fill_rect(dot, &dot_paint, Transform::identity(), None);
        }
    }
}
