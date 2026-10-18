// Clean borderless footer status bar: Bluetooth, Mobile App, GPS
// Displays minimalist vector icons and names:
// - White when connected (showing connected device name for Bluetooth)
// - Dim muted gray when disconnected

use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct FooterRenderer;

impl FooterRenderer {
    pub fn draw(pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState) {
        let white_color = Color::from_rgba8(255, 255, 255, 255);
        let dim_color = Color::from_rgba8(110, 110, 120, 255);

        // 1. Bluetooth item:
        // - Connected: white icon + connected device name
        // - Active/On (waiting): white icon + "Bluetooth"
        // - Disabled/Error: dim muted gray icon + "Bluetooth"
        let (bt_color, bt_label) = if let Some(ref name) = state.bt_device_name {
            (white_color, name.as_str())
        } else if state.bt_active {
            (white_color, "Bluetooth")
        } else {
            (dim_color, "Bluetooth")
        };

        // 2. App item
        let (app_color, app_label) = if state.app_connected {
            (white_color, "App Connected")
        } else {
            (dim_color, "App")
        };

        // 3. GPS item
        let (gps_color, gps_label) = if state.gps_connected {
            (white_color, "GPS")
        } else {
            (dim_color, "GPS")
        };

        let items = [
            (0, bt_color, bt_label),
            (1, app_color, app_label),
            (2, gps_color, gps_label),
        ];

        let font_size = 13.0;
        let icon_box_w = 12.0;
        let icon_text_gap = 7.0;
        let item_spacing = 45.0;
        let baseline_y = 458.0;
        let icon_cy = baseline_y - 4.5;

        // Measure widths to center the 3 items horizontally
        let mut item_widths = Vec::with_capacity(3);
        let mut total_w = 0.0;
        for (i, &(_, _, label)) in items.iter().enumerate() {
            let tw = fonts.measure_width(&fonts.bold, label, font_size);
            let iw = icon_box_w + icon_text_gap + tw;
            item_widths.push(iw);
            total_w += iw;
            if i > 0 {
                total_w += item_spacing;
            }
        }

        let mut cur_x = 640.0 - (total_w / 2.0);

        for (i, &(kind, color, label)) in items.iter().enumerate() {
            let iw = item_widths[i];
            let icon_cx = cur_x + (icon_box_w * 0.5);

            // Draw Icon
            match kind {
                0 => Self::draw_bluetooth_icon(pixmap, icon_cx, icon_cy, color),
                1 => Self::draw_app_icon(pixmap, icon_cx, icon_cy, color),
                _ => Self::draw_gps_icon(pixmap, icon_cx, icon_cy, color),
            }

            // Draw Label
            let text_x = cur_x + icon_box_w + icon_text_gap;
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                label,
                text_x,
                baseline_y,
                font_size,
                color,
            );

            cur_x += iw + item_spacing;
        }
    }

    /// Minimalist Bluetooth Rune Icon
    fn draw_bluetooth_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
        let mut pb = PathBuilder::new();
        // Vertical stem
        pb.move_to(cx, cy - 6.5);
        pb.line_to(cx, cy + 6.5);
        // Lower lobe and upper-left diagonal
        pb.line_to(cx + 3.2, cy + 3.2);
        pb.line_to(cx - 3.2, cy - 3.2);
        // Upper lobe and lower-left diagonal
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

    /// Minimalist Smartphone App Icon
    fn draw_app_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
        let w = 8.4;
        let h = 13.0;
        let r = 1.8;
        let x = cx - (w * 0.5);
        let y = cy - (h * 0.5);

        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);

        // Home indicator notch at bottom
        pb.move_to(cx - 1.6, cy + 4.2);
        pb.line_to(cx + 1.6, cy + 4.2);

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.3,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    /// Minimalist GPS Navigation Arrow Icon
    fn draw_gps_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
        let mut pb = PathBuilder::new();
        // Location compass arrow pointing upward
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
