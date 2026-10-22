// Center Panel: Real-time Bluetooth Subsystem Event & Diagnostics Log Viewer

use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::{Color, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct BtLogPanel;

impl BtLogPanel {
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
        let left_x = 420.0;
        let right_x = 860.0;
        let max_text_w = right_x - left_x - 10.0;

        // 1. Header: Bluetooth Icon, Title, and [B] Back hint
        let header_y = 58.0;
        let icon_cx = left_x + 8.0;
        let icon_cy = header_y - 4.5;
        let bt_cyan = Color::from_rgba8(0, 229, 255, 255);
        Self::draw_bt_icon(pixmap, icon_cx, icon_cy, bt_cyan);

        // Title
        fonts.draw_text(
            pixmap,
            &fonts.bold,
            "BLUETOOTH LOGS",
            left_x + 22.0,
            header_y,
            14.0,
            Color::from_rgba8(255, 255, 255, 255),
        );

        // [B] Close hint at top-right
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            "[B] Close",
            right_x - 52.0,
            header_y,
            11.0,
            Color::from_rgba8(140, 145, 155, 255),
        );

        // Header Divider Line
        let div_y = 72.0;
        let mut div_pb = PathBuilder::new();
        div_pb.move_to(left_x, div_y);
        div_pb.line_to(right_x, div_y);
        if let Some(path) = div_pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(35, 38, 48, 255));
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.0,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }

        // 2. Render Log Entries
        let max_visible_lines = 16;
        let line_height = 21.0;
        let start_y = 94.0;

        let logs = if state.bt_logs.is_empty() {
            &[
                "[System] Bluetooth log stream active. Waiting for events...".to_string(),
                "[System] Path: /var/log/icarmeter-bluetooth.log".to_string(),
                "[Hint] Press [B] to return to normal dashboard view".to_string(),
            ][..]
        } else {
            let start = state.bt_logs.len().saturating_sub(max_visible_lines);
            &state.bt_logs[start..]
        };

        for (idx, line) in logs.iter().enumerate() {
            if idx >= max_visible_lines {
                break;
            }
            let y = start_y + (idx as f32 * line_height);

            // Parse color and truncated text
            let (color, display_str) = Self::style_log_line(line, fonts, max_text_w);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                &display_str,
                left_x,
                y,
                11.0,
                color,
            );
        }
    }

    fn style_log_line(
        line: &str,
        fonts: &FontRenderer,
        max_w: f32,
    ) -> (Color, String) {
        let color = if line.contains("[Bluetooth-Init]") || line.contains("[System start]") {
            Color::from_rgba8(74, 222, 128, 255) // Bright Emerald Green
        } else if line.contains("[BTAgent]") || line.contains("[PAIR]") || line.contains("PIN") {
            Color::from_rgba8(251, 191, 36, 255) // Amber / Gold
        } else if line.contains("[BT-SPP]") || line.contains("[RFCOMM]") {
            Color::from_rgba8(56, 189, 248, 255) // Sky Blue
        } else if line.contains("[TPMS-BLE]") || line.contains("Discovered") {
            Color::from_rgba8(192, 132, 252, 255) // Violet Purple
        } else if line.contains("[BT-PROBE]") {
            Color::from_rgba8(45, 212, 191, 255) // Cyan Teal
        } else if line.contains("Warning") || line.contains("Warn") {
            Color::from_rgba8(251, 146, 60, 255) // Orange Warning
        } else if line.contains("Error") || line.contains("error") || line.contains("FAIL") {
            Color::from_rgba8(248, 113, 113, 255) // Coral Red Error
        } else {
            Color::from_rgba8(215, 220, 228, 255) // Crisp Neutral Light Gray
        };

        // Truncate line if it exceeds max width
        let font_size = 11.0;
        let mut display_str = line.to_string();
        if fonts.measure_width(&fonts.regular, &display_str, font_size) > max_w {
            while !display_str.is_empty()
                && fonts.measure_width(
                    &fonts.regular,
                    &format!("{}...", display_str),
                    font_size,
                ) > max_w
            {
                display_str.pop();
            }
            display_str.push_str("...");
        }

        (color, display_str)
    }

    fn draw_bt_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: Color) {
        let mut pb = PathBuilder::new();
        // Central vertical axis
        pb.move_to(cx, cy - 8.0);
        pb.line_to(cx, cy + 8.0);
        // Top right lobe
        pb.move_to(cx, cy);
        pb.line_to(cx + 4.5, cy - 4.0);
        pb.line_to(cx, cy - 8.0);
        // Bottom right lobe
        pb.move_to(cx, cy);
        pb.line_to(cx + 4.5, cy + 4.0);
        pb.line_to(cx, cy + 8.0);
        // Left diagonals
        pb.move_to(cx - 3.5, cy - 3.5);
        pb.line_to(cx + 3.5, cy + 3.5);
        pb.move_to(cx - 3.5, cy + 3.5);
        pb.line_to(cx + 3.5, cy - 3.5);

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.4,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    fn draw_circle(pixmap: &mut Pixmap, cx: f32, cy: f32, radius: f32, color: Color) {
        let mut pb = PathBuilder::new();
        let k = 0.55228475 * radius;
        pb.move_to(cx, cy - radius);
        pb.cubic_to(cx + k, cy - radius, cx + radius, cy - k, cx + radius, cy);
        pb.cubic_to(cx + radius, cy + k, cx + k, cy + radius, cx, cy + radius);
        pb.cubic_to(cx - k, cy + radius, cx - radius, cy + k, cx - radius, cy);
        pb.cubic_to(cx - radius, cy - k, cx - k, cy - radius, cx, cy - radius);

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }
}
