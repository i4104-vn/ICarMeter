// Compact connection footer: Bluetooth and GPS only.

use crate::configs::display::SCREEN_WIDTH;
use crate::models::vehicle::state::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

#[derive(Debug, Clone, Default)]
pub struct FooterRenderer;

impl FooterRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(
        &mut self,
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        state: &VehicleState,
        _dt: f32,
    ) {
        let white_color = Color::from_rgba8(255, 255, 255, 255);
        let dim_color = Color::from_rgba8(110, 110, 120, 255);
        let baseline_y = 458.0;

        let (bt_color, bt_label) = match (state.bt_device_name.as_deref(), state.bt_active) {
            (Some(name), _) => (white_color, name),
            (None, true) => (white_color, "Bluetooth"),
            (None, false) => (dim_color, "Bluetooth"),
        };
        let (gps_color, gps_label) = if state.gps_connected {
            (white_color, "GPS")
        } else {
            (dim_color, "GPS")
        };

        let items = [(0, bt_color, bt_label), (1, gps_color, gps_label)];
        let font_size = 13.0;
        let icon_box_w = 12.0;
        let icon_text_gap = 7.0;
        let item_spacing = 42.0;
        let icon_cy = baseline_y - 4.5;

        let mut item_widths = Vec::with_capacity(items.len());
        let mut total_w = 0.0;
        for (i, &(_, _, label)) in items.iter().enumerate() {
            let text_width = fonts.measure_width(&fonts.bold, label, font_size);
            let item_width = icon_box_w + icon_text_gap + text_width;
            item_widths.push(item_width);
            total_w += item_width;
            if i > 0 {
                total_w += item_spacing;
            }
        }

        let mut cur_x = (SCREEN_WIDTH as f32 - total_w) / 2.0;
        for (i, &(kind, color, label)) in items.iter().enumerate() {
            let item_width = item_widths[i];
            let icon_cx = cur_x + (icon_box_w * 0.5);

            match kind {
                0 => Self::draw_bluetooth_icon(pixmap, icon_cx, icon_cy, color),
                _ => Self::draw_gps_icon(pixmap, icon_cx, icon_cy, color),
            }

            fonts.draw_text(
                pixmap,
                &fonts.bold,
                label,
                cur_x + icon_box_w + icon_text_gap,
                baseline_y,
                font_size,
                color,
            );
            cur_x += item_width + item_spacing;
        }

        let version_label = concat!("v", env!("CARGO_PKG_VERSION"));
        let version_size = 10.0;
        let version_width = fonts.measure_width(&fonts.regular, version_label, version_size);
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            version_label,
            SCREEN_WIDTH as f32 - 20.0 - version_width,
            baseline_y,
            version_size,
            dim_color,
        );
    }

    fn draw_bluetooth_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
        let mut pb = PathBuilder::new();
        pb.move_to(cx, cy - 6.5);
        pb.line_to(cx, cy + 6.5);
        pb.line_to(cx + 3.2, cy + 3.2);
        pb.line_to(cx - 3.2, cy - 3.2);
        pb.move_to(cx - 3.2, cy + 3.2);
        pb.line_to(cx + 3.2, cy - 3.2);
        pb.line_to(cx, cy - 6.5);

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.4,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    fn draw_gps_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
        let mut pb = PathBuilder::new();
        pb.move_to(cx, cy - 6.5);
        pb.line_to(cx + 4.6, cy + 5.5);
        pb.line_to(cx, cy + 2.8);
        pb.line_to(cx - 4.6, cy + 5.5);
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
}
