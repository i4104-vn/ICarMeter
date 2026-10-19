// Clean borderless footer status bar: Bluetooth, Mobile App, GPS
// Displays minimalist vector icons and names:
// - White when connected (showing connected device name for Bluetooth)
// - Dim muted gray when disconnected

use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

#[derive(Debug, Clone)]
pub struct FooterRenderer {
    current_event: String,
    prev_event: Option<String>,
    slide_anim: f32,
}

impl Default for FooterRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl FooterRenderer {
    pub fn new() -> Self {
        Self {
            current_event: "Standby".to_string(),
            prev_event: None,
            slide_anim: 1.0,
        }
    }

    pub fn draw(&mut self, pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState, dt: f32) {
        let white_color = Color::from_rgba8(255, 255, 255, 255);
        let dim_color = Color::from_rgba8(110, 110, 120, 255);
        let baseline_y = 458.0;

        // ─── 1. WebSocket Event Log with Slide-Up Animation (Bottom-Left) ────
        // Strictly 1-line event ticker: only shows the concise event name, slides up on new event
        let new_event = state.ws_event_log.as_deref().unwrap_or("Standby");
        if new_event != self.current_event {
            self.prev_event = Some(self.current_event.clone());
            self.current_event = new_event.to_string();
            self.slide_anim = 0.0; // Trigger slide-up animation
        }

        if self.slide_anim < 1.0 {
            self.slide_anim = (self.slide_anim + dt * 4.0).min(1.0);
        }

        // Hard vertical clip range: strictly 1 line height (15px slot)
        let clip_min_y = (baseline_y - 12.0) as i32;
        let clip_max_y = (baseline_y + 3.0) as i32;

        let t = 1.0 - (1.0 - self.slide_anim).powi(3); // ease-out cubic
        let slide_dist = 15.0;
        let event_x = 28.0;

        if self.slide_anim < 1.0 {
            // 1. Previous event slides UP and exits the top of the single-line slot
            if let Some(ref prev) = self.prev_event {
                let prev_y = baseline_y - t * slide_dist;
                let prev_alpha = ((1.0 - t) * 150.0) as u8;
                if prev_alpha > 0 {
                    fonts.draw_text_clipped_y(
                        pixmap,
                        &fonts.regular,
                        prev,
                        event_x,
                        prev_y,
                        11.5,
                        Color::from_rgba8(115, 120, 132, prev_alpha),
                        clip_min_y,
                        clip_max_y,
                    );
                }
            }

            // 2. Incoming event slides UP from below and enters the single-line slot
            let cur_y = baseline_y + (1.0 - t) * slide_dist;
            let cur_alpha = (t * 180.0) as u8;
            fonts.draw_text_clipped_y(
                pixmap,
                &fonts.regular,
                &self.current_event,
                event_x,
                cur_y,
                11.5,
                Color::from_rgba8(120, 125, 138, cur_alpha),
                clip_min_y,
                clip_max_y,
            );
        } else {
            // At rest: strictly 1 single line, static and crystal clear dim text
            fonts.draw_text_clipped_y(
                pixmap,
                &fonts.regular,
                &self.current_event,
                event_x,
                baseline_y,
                11.5,
                Color::from_rgba8(120, 125, 138, 180),
                clip_min_y,
                clip_max_y,
            );
        }

        // ─── 2. Center Status Icons: Wi-Fi, Bluetooth, GPS ───────────────────
        // 1. Wi-Fi Hotspot item
        let (wifi_color, wifi_label) = if state.wifi_active {
            (white_color, "Wi-Fi")
        } else {
            (dim_color, "Wi-Fi")
        };

        // 2. Bluetooth item (pure Bluetooth / BLE sensors, isolated from WebSocket)
        let (bt_color, bt_label) = if let Some(ref name) = state.bt_device_name {
            (white_color, name.as_str())
        } else if state.bt_active {
            (white_color, "Bluetooth")
        } else {
            (dim_color, "Bluetooth")
        };

        // 3. GPS item
        let (gps_color, gps_label) = if state.gps_connected {
            (white_color, "GPS")
        } else {
            (dim_color, "GPS")
        };

        let items = [
            (0, wifi_color, wifi_label),
            (1, bt_color, bt_label),
            (3, gps_color, gps_label),
        ];

        let font_size = 13.0;
        let icon_box_w = 12.0;
        let icon_text_gap = 7.0;
        let item_spacing = 42.0;
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
                0 => Self::draw_wifi_icon(pixmap, icon_cx, icon_cy, color),
                1 => Self::draw_bluetooth_icon(pixmap, icon_cx, icon_cy, color),
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

    /// Minimalist filled circle dot
    #[allow(dead_code)]
    fn draw_dot(pixmap: &mut Pixmap, cx: f32, cy: f32, r: f32, color: Color) {
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, r);
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
    #[allow(dead_code)]
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

    /// Minimalist Wi-Fi Hotspot Icon
    fn draw_wifi_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
        let mut pb = PathBuilder::new();
        // 1. Center base dot
        pb.move_to(cx - 1.2, cy + 4.6);
        pb.line_to(cx + 1.2, cy + 4.6);

        // 2. Middle arc
        pb.move_to(cx - 3.6, cy + 1.0);
        pb.quad_to(cx, cy - 2.2, cx + 3.6, cy + 1.0);

        // 3. Top outer arc
        pb.move_to(cx - 6.5, cy - 3.0);
        pb.quad_to(cx, cy - 7.0, cx + 6.5, cy - 3.0);

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
