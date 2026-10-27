// 4 Side Wheels rendering with alert red fill & glow for low tire pressure

use crate::configs::colors;
use crate::models::tpms::pressures::TirePressures;
use crate::render::core::create_rounded_rect;
use tiny_skia::{Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub const WHEEL_WIDTH: f32 = 15.0;
pub const WHEEL_HEIGHT: f32 = 36.0;
pub const WHEEL_RADIUS: f32 = 4.0;

pub struct Wheels;

impl Wheels {
    pub fn draw(pixmap: &mut Pixmap, cx: f32, cy: f32, tires: &TirePressures) {
        let body_half_w = 44.0;
        let fl_x = cx - body_half_w - WHEEL_WIDTH - 3.0; // 178.0
        let fr_x = cx + body_half_w + 3.0; // 287.0
        let front_y = cy - 58.0;
        let rear_y = cy + 22.0;

        let wheel_items = [
            (fl_x, front_y, tires.fl, tires.fl_temp),
            (fr_x, front_y, tires.fr, tires.fr_temp),
            (fl_x, rear_y, tires.rl, tires.rl_temp),
            (fr_x, rear_y, tires.rr, tires.rr_temp),
        ];

        let alert_red = colors::alert_red();
        let alert_red_fill = colors::alert_red_fill();
        let alert_red_glow = colors::alert_red_glow();
        let normal_wheel_fill = Color::from_rgba8(22, 22, 24, 255);
        let normal_wheel_border = Color::from_rgba8(75, 75, 80, 255);

        for &(wx, wy, pressure, temp) in &wheel_items {
            let is_alert = tires.is_low(pressure)
                || tires.is_high_temp(temp)
                || tires.is_disconnected(pressure);
            let wp = create_rounded_rect(wx, wy, WHEEL_WIDTH, WHEEL_HEIGHT, WHEEL_RADIUS);

            if is_alert {
                // RED ALERT: Fill tire with warning red & glow
                let mut glow_paint = Paint::default();
                glow_paint.set_color(alert_red_glow);
                glow_paint.anti_alias = true;
                let glow_stroke = Stroke {
                    width: 4.5,
                    line_cap: LineCap::Round,
                    ..Default::default()
                };
                pixmap.stroke_path(&wp, &glow_paint, &glow_stroke, Transform::identity(), None);

                let mut fill_paint = Paint::default();
                fill_paint.set_color(alert_red_fill);
                fill_paint.anti_alias = true;
                pixmap.fill_path(
                    &wp,
                    &fill_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(alert_red);
                stroke_paint.anti_alias = true;
                let stroke = Stroke {
                    width: 1.8,
                    line_cap: LineCap::Round,
                    ..Default::default()
                };
                pixmap.stroke_path(&wp, &stroke_paint, &stroke, Transform::identity(), None);

                // Warning indicator line inside wheel
                let mut rim_pb = PathBuilder::new();
                rim_pb.move_to(wx + WHEEL_WIDTH * 0.5, wy + 6.0);
                rim_pb.line_to(wx + WHEEL_WIDTH * 0.5, wy + WHEEL_HEIGHT - 6.0);
                if let Some(rim_path) = rim_pb.finish() {
                    let mut rim_paint = Paint::default();
                    rim_paint.set_color(Color::from_rgba8(255, 255, 255, 220));
                    let rim_stroke = Stroke {
                        width: 1.5,
                        ..Default::default()
                    };
                    pixmap.stroke_path(
                        &rim_path,
                        &rim_paint,
                        &rim_stroke,
                        Transform::identity(),
                        None,
                    );
                }
            } else {
                // NORMAL TIRE: Clean dark rubber with cyan border
                let mut fill_paint = Paint::default();
                fill_paint.set_color(normal_wheel_fill);
                fill_paint.anti_alias = true;
                pixmap.fill_path(
                    &wp,
                    &fill_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(normal_wheel_border);
                stroke_paint.anti_alias = true;
                let stroke = Stroke {
                    width: 1.5,
                    line_cap: LineCap::Round,
                    ..Default::default()
                };
                pixmap.stroke_path(&wp, &stroke_paint, &stroke, Transform::identity(), None);

                // Wheel rim center line
                let mut rim_pb = PathBuilder::new();
                rim_pb.move_to(wx + WHEEL_WIDTH * 0.5, wy + 7.0);
                rim_pb.line_to(wx + WHEEL_WIDTH * 0.5, wy + WHEEL_HEIGHT - 7.0);
                if let Some(rim_path) = rim_pb.finish() {
                    let rim_stroke = Stroke {
                        width: 1.0,
                        ..Default::default()
                    };
                    pixmap.stroke_path(
                        &rim_path,
                        &stroke_paint,
                        &rim_stroke,
                        Transform::identity(),
                        None,
                    );
                }
            }
        }
    }
}
