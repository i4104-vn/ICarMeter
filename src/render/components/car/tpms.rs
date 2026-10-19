// TPMS overlay: tire pressure & temperature readouts with horizontal separator and leader lines to wheels

use super::wheels::{WHEEL_HEIGHT, WHEEL_WIDTH};
use crate::configs::colors;
use crate::models::tpms::TirePressures;
use crate::render::core::{create_rounded_rect, FontRenderer};
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct TpmsOverlay;

impl TpmsOverlay {
    pub fn draw(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        cx: f32,
        cy: f32,
        tires: &TirePressures,
    ) {
        let body_half_w = 44.0;
        let fl_x = cx - body_half_w - WHEEL_WIDTH - 3.0; // 160.0 when cx = 222
        let fr_x = cx + body_half_w + 3.0; // 269.0 when cx = 222
        let front_y = cy - 58.0;
        let rear_y = cy + 22.0;

        let stroke_config = Stroke {
            width: 1.2,
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: None,
        };

        let alert_red = Color::from_rgba8(240, 50, 50, 230);
        let normal_line_color = colors::accent_cyan();
        let muted_temp_color = Color::from_rgba8(140, 145, 155, 190);

        // ─────────────────────────────────────────────────────────────────────
        // 1. Top-Left Tire (Front-Left)
        // ─────────────────────────────────────────────────────────────────────
        let fl_disconnected = tires.is_disconnected(tires.fl);
        let fl_is_low = tires.is_low(tires.fl);
        let fl_is_hot = tires.is_high_temp(tires.fl_temp);
        let fl_alert = fl_disconnected || fl_is_low || fl_is_hot;
        let fl_line_color = if fl_alert { alert_red } else { normal_line_color };

        if fl_disconnected {
            Self::draw_error_badge(pixmap, fonts, 60.0, 110.0, 76.0, 20.0, alert_red);
        } else {
            let fl_p_color = if fl_is_low { alert_red } else { colors::text_primary() };
            let fl_u_color = if fl_is_low { alert_red } else { colors::text_muted() };
            let fl_p_val = format!("{:.1}", tires.fl);
            let fl_p_w = fonts.measure_width(&fonts.bold, &fl_p_val, 22.0);

            fonts.draw_text(pixmap, &fonts.bold, &fl_p_val, 60.0, 126.0, 22.0, fl_p_color);
            fonts.draw_text(pixmap, &fonts.regular, " bar", 60.0 + fl_p_w, 126.0, 13.0, fl_u_color);
        }

        // Horizontal Separator & Leader Line to Front-Left Wheel
        let fl_target_x = fl_x - 3.0;
        let fl_target_y = front_y + (WHEEL_HEIGHT / 2.0);
        let mut pb_fl = PathBuilder::new();
        pb_fl.move_to(60.0, 135.0);
        pb_fl.line_to(144.0, 135.0);
        pb_fl.line_to(fl_target_x, fl_target_y);
        if let Some(path) = pb_fl.finish() {
            let mut lp = Paint::default();
            lp.set_color(fl_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }
        Self::draw_circle(pixmap, fl_target_x, fl_target_y, 2.2, fl_line_color);

        // Temperature (Bottom)
        if fl_disconnected {
            fonts.draw_text(pixmap, &fonts.regular, "-- °C", 60.0, 153.0, 13.5, muted_temp_color);
        } else {
            let fl_t_color = if fl_is_hot { alert_red } else { colors::text_muted() };
            let fl_t_val = format!("{:.0}°C", tires.fl_temp);
            fonts.draw_text(pixmap, &fonts.regular, &fl_t_val, 60.0, 153.0, 14.0, fl_t_color);
        }

        // ─────────────────────────────────────────────────────────────────────
        // 2. Top-Right Tire (Front-Right)
        // ─────────────────────────────────────────────────────────────────────
        let fr_disconnected = tires.is_disconnected(tires.fr);
        let fr_is_low = tires.is_low(tires.fr);
        let fr_is_hot = tires.is_high_temp(tires.fr_temp);
        let fr_alert = fr_disconnected || fr_is_low || fr_is_hot;
        let fr_line_color = if fr_alert { alert_red } else { normal_line_color };
        let fr_block_right = 380.0;

        if fr_disconnected {
            Self::draw_error_badge(pixmap, fonts, fr_block_right - 76.0, 110.0, 76.0, 20.0, alert_red);
        } else {
            let fr_p_color = if fr_is_low { alert_red } else { colors::text_primary() };
            let fr_u_color = if fr_is_low { alert_red } else { colors::text_muted() };
            let fr_p_val = format!("{:.1}", tires.fr);
            let fr_p_w = fonts.measure_width(&fonts.bold, &fr_p_val, 22.0);
            let fr_u_w = fonts.measure_width(&fonts.regular, " bar", 13.0);
            let fr_p_start = fr_block_right - (fr_p_w + fr_u_w);

            fonts.draw_text(pixmap, &fonts.bold, &fr_p_val, fr_p_start, 126.0, 22.0, fr_p_color);
            fonts.draw_text(pixmap, &fonts.regular, " bar", fr_p_start + fr_p_w, 126.0, 13.0, fr_u_color);
        }

        // Horizontal Separator & Leader Line to Front-Right Wheel
        let fr_target_x = fr_x + WHEEL_WIDTH + 3.0;
        let fr_target_y = front_y + (WHEEL_HEIGHT / 2.0);
        let mut pb_fr = PathBuilder::new();
        pb_fr.move_to(fr_block_right, 135.0);
        pb_fr.line_to(296.0, 135.0);
        pb_fr.line_to(fr_target_x, fr_target_y);
        if let Some(path) = pb_fr.finish() {
            let mut lp = Paint::default();
            lp.set_color(fr_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }
        Self::draw_circle(pixmap, fr_target_x, fr_target_y, 2.2, fr_line_color);

        // Temperature (Bottom)
        if fr_disconnected {
            let t_w = fonts.measure_width(&fonts.regular, "-- °C", 13.5);
            fonts.draw_text(pixmap, &fonts.regular, "-- °C", fr_block_right - t_w, 153.0, 13.5, muted_temp_color);
        } else {
            let fr_t_color = if fr_is_hot { alert_red } else { colors::text_muted() };
            let fr_t_val = format!("{:.0}°C", tires.fr_temp);
            let fr_t_w = fonts.measure_width(&fonts.regular, &fr_t_val, 14.0);
            fonts.draw_text(pixmap, &fonts.regular, &fr_t_val, fr_block_right - fr_t_w, 153.0, 14.0, fr_t_color);
        }

        // ─────────────────────────────────────────────────────────────────────
        // 3. Bottom-Left Tire (Rear-Left)
        // ─────────────────────────────────────────────────────────────────────
        let rl_disconnected = tires.is_disconnected(tires.rl);
        let rl_is_low = tires.is_low(tires.rl);
        let rl_is_hot = tires.is_high_temp(tires.rl_temp);
        let rl_alert = rl_disconnected || rl_is_low || rl_is_hot;
        let rl_line_color = if rl_alert { alert_red } else { normal_line_color };

        if rl_disconnected {
            Self::draw_error_badge(pixmap, fonts, 60.0, 312.0, 76.0, 20.0, alert_red);
        } else {
            let rl_p_color = if rl_is_low { alert_red } else { colors::text_primary() };
            let rl_u_color = if rl_is_low { alert_red } else { colors::text_muted() };
            let rl_p_val = format!("{:.1}", tires.rl);
            let rl_p_w = fonts.measure_width(&fonts.bold, &rl_p_val, 22.0);

            fonts.draw_text(pixmap, &fonts.bold, &rl_p_val, 60.0, 328.0, 22.0, rl_p_color);
            fonts.draw_text(pixmap, &fonts.regular, " bar", 60.0 + rl_p_w, 328.0, 13.0, rl_u_color);
        }

        // Horizontal Separator & Leader Line to Rear-Left Wheel
        let rl_target_x = fl_x - 3.0;
        let rl_target_y = rear_y + (WHEEL_HEIGHT / 2.0);
        let mut pb_rl = PathBuilder::new();
        pb_rl.move_to(60.0, 337.0);
        pb_rl.line_to(144.0, 337.0);
        pb_rl.line_to(rl_target_x, rl_target_y);
        if let Some(path) = pb_rl.finish() {
            let mut lp = Paint::default();
            lp.set_color(rl_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }
        Self::draw_circle(pixmap, rl_target_x, rl_target_y, 2.2, rl_line_color);

        // Temperature (Bottom)
        if rl_disconnected {
            fonts.draw_text(pixmap, &fonts.regular, "-- °C", 60.0, 355.0, 13.5, muted_temp_color);
        } else {
            let rl_t_color = if rl_is_hot { alert_red } else { colors::text_muted() };
            let rl_t_val = format!("{:.0}°C", tires.rl_temp);
            fonts.draw_text(pixmap, &fonts.regular, &rl_t_val, 60.0, 355.0, 14.0, rl_t_color);
        }

        // ─────────────────────────────────────────────────────────────────────
        // 4. Bottom-Right Tire (Rear-Right)
        // ─────────────────────────────────────────────────────────────────────
        let rr_disconnected = tires.is_disconnected(tires.rr);
        let rr_is_low = tires.is_low(tires.rr);
        let rr_is_hot = tires.is_high_temp(tires.rr_temp);
        let rr_alert = rr_disconnected || rr_is_low || rr_is_hot;
        let rr_line_color = if rr_alert { alert_red } else { normal_line_color };
        let rr_block_right = 380.0;

        if rr_disconnected {
            Self::draw_error_badge(pixmap, fonts, rr_block_right - 76.0, 312.0, 76.0, 20.0, alert_red);
        } else {
            let rr_p_color = if rr_is_low { alert_red } else { colors::text_primary() };
            let rr_u_color = if rr_is_low { alert_red } else { colors::text_muted() };
            let rr_p_val = format!("{:.1}", tires.rr);
            let rr_p_w = fonts.measure_width(&fonts.bold, &rr_p_val, 22.0);
            let rr_u_w = fonts.measure_width(&fonts.regular, " bar", 13.0);
            let rr_p_start = rr_block_right - (rr_p_w + rr_u_w);

            fonts.draw_text(pixmap, &fonts.bold, &rr_p_val, rr_p_start, 328.0, 22.0, rr_p_color);
            fonts.draw_text(pixmap, &fonts.regular, " bar", rr_p_start + rr_p_w, 328.0, 13.0, rr_u_color);
        }

        // Horizontal Separator & Leader Line to Rear-Right Wheel
        let rr_target_x = fr_x + WHEEL_WIDTH + 3.0;
        let rr_target_y = rear_y + (WHEEL_HEIGHT / 2.0);
        let mut pb_rr = PathBuilder::new();
        pb_rr.move_to(rr_block_right, 337.0);
        pb_rr.line_to(296.0, 337.0);
        pb_rr.line_to(rr_target_x, rr_target_y);
        if let Some(path) = pb_rr.finish() {
            let mut lp = Paint::default();
            lp.set_color(rr_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }
        Self::draw_circle(pixmap, rr_target_x, rr_target_y, 2.2, rr_line_color);

        // Temperature (Bottom)
        if rr_disconnected {
            let t_w = fonts.measure_width(&fonts.regular, "-- °C", 13.5);
            fonts.draw_text(pixmap, &fonts.regular, "-- °C", rr_block_right - t_w, 355.0, 13.5, muted_temp_color);
        } else {
            let rr_t_color = if rr_is_hot { alert_red } else { colors::text_muted() };
            let rr_t_val = format!("{:.0}°C", tires.rr_temp);
            let rr_t_w = fonts.measure_width(&fonts.regular, &rr_t_val, 14.0);
            fonts.draw_text(pixmap, &fonts.regular, &rr_t_val, rr_block_right - rr_t_w, 355.0, 14.0, rr_t_color);
        }
    }

    /// Renders a modern automotive frosted glassmorphism error badge: [ ⚠  ERROR ]
    fn draw_error_badge(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
    ) {
        // 1. Frosted crimson capsule background
        let badge_rect = create_rounded_rect(x, y, w, h, 4.0);
        let mut bg_paint = Paint::default();
        bg_paint.set_color(Color::from_rgba8(235, 35, 35, 26)); // Soft dark crimson translucent tint
        bg_paint.anti_alias = true;
        pixmap.fill_path(
            &badge_rect,
            &bg_paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );

        // 2. Crisp hairline border
        let mut stroke_paint = Paint::default();
        stroke_paint.set_color(Color::from_rgba8(235, 55, 55, 140)); // Refined alert border
        stroke_paint.anti_alias = true;
        let border_stroke = Stroke {
            width: 1.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };
        pixmap.stroke_path(
            &badge_rect,
            &stroke_paint,
            &border_stroke,
            Transform::identity(),
            None,
        );

        // 3. Crisp Vector Warning Triangle ⚠
        let icon_cx = x + 13.0;
        let icon_cy = y + (h * 0.5);

        let mut tri_pb = PathBuilder::new();
        tri_pb.move_to(icon_cx, icon_cy - 4.5);
        tri_pb.line_to(icon_cx + 4.8, icon_cy + 4.0);
        tri_pb.line_to(icon_cx - 4.8, icon_cy + 4.0);
        tri_pb.close();
        if let Some(tri_path) = tri_pb.finish() {
            let mut tri_paint = Paint::default();
            tri_paint.set_color(color);
            tri_paint.anti_alias = true;
            let tri_stroke = Stroke {
                width: 1.2,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(
                &tri_path,
                &tri_paint,
                &tri_stroke,
                Transform::identity(),
                None,
            );
        }

        // Exclamation mark stem
        let mut exc_pb = PathBuilder::new();
        exc_pb.move_to(icon_cx, icon_cy - 2.0);
        exc_pb.line_to(icon_cx, icon_cy + 0.8);
        if let Some(exc_path) = exc_pb.finish() {
            let mut exc_paint = Paint::default();
            exc_paint.set_color(color);
            exc_paint.anti_alias = true;
            let exc_stroke = Stroke {
                width: 1.2,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(
                &exc_path,
                &exc_paint,
                &exc_stroke,
                Transform::identity(),
                None,
            );
        }
        // Exclamation mark dot
        Self::draw_circle(pixmap, icon_cx, icon_cy + 2.7, 0.7, color);

        // 4. Bold ERROR text inside capsule
        fonts.draw_text(
            pixmap,
            &fonts.bold,
            "ERROR",
            x + 22.5,
            y + 14.5,
            11.5,
            color,
        );
    }

    /// Minimalist filled circular dot helper
    fn draw_circle(pixmap: &mut Pixmap, cx: f32, cy: f32, r: f32, color: Color) {
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
}
