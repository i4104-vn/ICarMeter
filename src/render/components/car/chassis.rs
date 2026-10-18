// Vector car chassis: aerodynamic body silhouette, headlights, taillights, and glass roof

use crate::render::core::create_rounded_rect;
use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform,
};

pub struct CarChassis;

impl CarChassis {
    pub fn draw(pixmap: &mut Pixmap, cx: f32, cy: f32) {
        let body_w = 88.0;
        let body_h = 172.0;
        let body_x = cx - (body_w / 2.0);
        let body_y = cy - (body_h / 2.0);

        // Side mirrors
        let mirror_w = 8.0;
        let mirror_h = 14.0;
        let mirror_y = cy - 32.0;
        let lm = create_rounded_rect(body_x - mirror_w + 1.0, mirror_y, mirror_w, mirror_h, 3.0);
        let rm = create_rounded_rect(body_x + body_w - 1.0, mirror_y, mirror_w, mirror_h, 3.0);

        let mut mirror_paint = Paint::default();
        mirror_paint.set_color(Color::from_rgba8(36, 36, 40, 255));
        mirror_paint.anti_alias = true;
        pixmap.fill_path(
            &lm,
            &mirror_paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        pixmap.fill_path(
            &rm,
            &mirror_paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );

        // Body outline path
        let body_path = Self::create_car_body_path(body_x, body_y, body_w, body_h);

        // Body fill (Deep luxury dark obsidian / metallic black)
        let mut body_fill = Paint::default();
        body_fill.set_color(Color::from_rgba8(18, 18, 20, 255));
        body_fill.anti_alias = true;
        pixmap.fill_path(
            &body_path,
            &body_fill,
            FillRule::Winding,
            Transform::identity(),
            None,
        );

        // Body border stroke
        let mut body_stroke_paint = Paint::default();
        body_stroke_paint.set_color(Color::from_rgba8(70, 70, 76, 255));
        body_stroke_paint.anti_alias = true;
        let body_stroke = Stroke {
            width: 2.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };
        pixmap.stroke_path(
            &body_path,
            &body_stroke_paint,
            &body_stroke,
            Transform::identity(),
            None,
        );

        // Front Windshield (Tinted glass arc)
        let mut pb_fw = PathBuilder::new();
        pb_fw.move_to(body_x + 11.0, cy - 25.0);
        pb_fw.quad_to(cx, cy - 38.0, body_x + body_w - 11.0, cy - 25.0);
        if let Some(fw_path) = pb_fw.finish() {
            let mut glass_paint = Paint::default();
            glass_paint.set_color(Color::from_rgba8(140, 145, 150, 180));
            glass_paint.anti_alias = true;
            let glass_stroke = Stroke {
                width: 2.0,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(
                &fw_path,
                &glass_paint,
                &glass_stroke,
                Transform::identity(),
                None,
            );
        }

        // Panoramic Sunroof outline
        let roof_w = body_w - 26.0;
        let roof_h = 58.0;
        let roof_x = cx - (roof_w / 2.0);
        let roof_y = cy - 20.0;
        let roof_path = create_rounded_rect(roof_x, roof_y, roof_w, roof_h, 8.0);
        let mut roof_paint = Paint::default();
        roof_paint.set_color(Color::from_rgba8(42, 42, 46, 140));
        let roof_stroke = Stroke {
            width: 1.2,
            ..Default::default()
        };
        pixmap.stroke_path(
            &roof_path,
            &roof_paint,
            &roof_stroke,
            Transform::identity(),
            None,
        );

        // Rear Windshield Arc
        let mut pb_rw = PathBuilder::new();
        pb_rw.move_to(body_x + 13.0, cy + 44.0);
        pb_rw.quad_to(cx, cy + 36.0, body_x + body_w - 13.0, cy + 44.0);
        if let Some(rw_path) = pb_rw.finish() {
            let mut glass_paint = Paint::default();
            glass_paint.set_color(Color::from_rgba8(140, 145, 150, 160));
            glass_paint.anti_alias = true;
            let glass_stroke = Stroke {
                width: 1.6,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(
                &rw_path,
                &glass_paint,
                &glass_stroke,
                Transform::identity(),
                None,
            );
        }

        // Front Headlights (Twin sleek LED lightbars)
        let mut led_paint = Paint::default();
        led_paint.set_color(Color::from_rgba8(165, 240, 255, 255));
        led_paint.anti_alias = true;
        let led_stroke = Stroke {
            width: 2.5,
            line_cap: LineCap::Round,
            ..Default::default()
        };

        let mut pb_led = PathBuilder::new();
        pb_led.move_to(body_x + 9.0, body_y + 12.0);
        pb_led.line_to(body_x + 24.0, body_y + 6.0);
        pb_led.move_to(body_x + body_w - 9.0, body_y + 12.0);
        pb_led.line_to(body_x + body_w - 24.0, body_y + 6.0);
        if let Some(led_path) = pb_led.finish() {
            pixmap.stroke_path(
                &led_path,
                &led_paint,
                &led_stroke,
                Transform::identity(),
                None,
            );
        }

        // Rear Taillights (Sleek red LED accents)
        let mut tail_paint = Paint::default();
        tail_paint.set_color(Color::from_rgba8(240, 45, 45, 240));
        tail_paint.anti_alias = true;
        let tail_stroke = Stroke {
            width: 2.2,
            line_cap: LineCap::Round,
            ..Default::default()
        };

        let mut pb_tail = PathBuilder::new();
        pb_tail.move_to(body_x + 12.0, body_y + body_h - 6.0);
        pb_tail.line_to(body_x + 25.0, body_y + body_h - 4.0);
        pb_tail.move_to(body_x + body_w - 12.0, body_y + body_h - 6.0);
        pb_tail.line_to(body_x + body_w - 25.0, body_y + body_h - 4.0);
        if let Some(tail_path) = pb_tail.finish() {
            pixmap.stroke_path(
                &tail_path,
                &tail_paint,
                &tail_stroke,
                Transform::identity(),
                None,
            );
        }
    }

    /// Helper to generate a stylized car body contour path
    fn create_car_body_path(x: f32, y: f32, w: f32, h: f32) -> tiny_skia::Path {
        let mut pb = PathBuilder::new();
        let r_front = 24.0;
        let r_rear = 16.0;

        // Front bumper
        pb.move_to(x + r_front, y);
        pb.line_to(x + w - r_front, y);
        pb.quad_to(x + w, y, x + w, y + r_front);

        // Right flank
        pb.line_to(x + w, y + h - r_rear);
        pb.quad_to(x + w, y + h, x + w - r_rear, y + h);

        // Rear bumper
        pb.line_to(x + r_rear, y + h);
        pb.quad_to(x, y + h, x, y + h - r_rear);

        // Left flank
        pb.line_to(x, y + r_front);
        pb.quad_to(x, y, x + r_front, y);

        pb.finish().expect("Valid car body path")
    }
}
