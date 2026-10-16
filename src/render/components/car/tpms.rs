// TPMS overlay: tire pressure readouts and leader lines to wheels

use super::wheels::{WHEEL_HEIGHT, WHEEL_WIDTH};
use crate::configs::colors;
use crate::models::tpms::TirePressures;
use crate::render::core::FontRenderer;
use tiny_skia::{LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

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
            width: 1.8,
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: None,
        };

        let alert_red = colors::alert_red();

        // 1. Top-Left Tire (Front-Left)
        let fl_is_low = TirePressures::is_low(tires.fl);
        let fl_color = if fl_is_low {
            alert_red
        } else {
            colors::text_primary()
        };
        let fl_unit_color = if fl_is_low {
            alert_red
        } else {
            colors::accent_cyan()
        };
        let fl_val = format!("{:.1}", tires.fl);
        fonts.draw_text(pixmap, &fonts.bold, &fl_val, 65.0, 135.0, 28.0, fl_color);
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            "bar",
            65.0,
            160.0,
            20.0,
            fl_unit_color,
        );

        let mut pb_fl = PathBuilder::new();
        pb_fl.move_to(110.0, 142.0);
        pb_fl.line_to(138.0, 142.0);
        pb_fl.line_to(fl_x - 3.0, front_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_fl.finish() {
            let mut lp = Paint::default();
            lp.set_color(fl_unit_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }

        // 2. Top-Right Tire (Front-Right)
        let fr_is_low = TirePressures::is_low(tires.fr);
        let fr_color = if fr_is_low {
            alert_red
        } else {
            colors::text_primary()
        };
        let fr_unit_color = if fr_is_low {
            alert_red
        } else {
            colors::accent_cyan()
        };
        let fr_val = format!("{:.1}", tires.fr);
        let fr_val_w = fonts.measure_width(&fonts.bold, &fr_val, 28.0);
        let fr_unit_w = fonts.measure_width(&fonts.regular, "bar", 20.0);
        fonts.draw_text(
            pixmap,
            &fonts.bold,
            &fr_val,
            380.0 - fr_val_w,
            135.0,
            28.0,
            fr_color,
        );
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            "bar",
            380.0 - fr_unit_w,
            160.0,
            20.0,
            fr_unit_color,
        );

        let mut pb_fr = PathBuilder::new();
        pb_fr.move_to(335.0, 142.0);
        pb_fr.line_to(307.0, 142.0);
        pb_fr.line_to(fr_x + WHEEL_WIDTH + 3.0, front_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_fr.finish() {
            let mut lp = Paint::default();
            lp.set_color(fr_unit_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }

        // 3. Bottom-Left Tire (Rear-Left)
        let rl_is_low = TirePressures::is_low(tires.rl);
        let rl_color = if rl_is_low {
            alert_red
        } else {
            colors::text_primary()
        };
        let rl_unit_color = if rl_is_low {
            alert_red
        } else {
            colors::accent_cyan()
        };
        let rl_val = format!("{:.1}", tires.rl);
        fonts.draw_text(pixmap, &fonts.bold, &rl_val, 65.0, 335.0, 28.0, rl_color);
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            "bar",
            65.0,
            360.0,
            20.0,
            rl_unit_color,
        );

        let mut pb_rl = PathBuilder::new();
        pb_rl.move_to(110.0, 345.0);
        pb_rl.line_to(138.0, 345.0);
        pb_rl.line_to(fl_x - 3.0, rear_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_rl.finish() {
            let mut lp = Paint::default();
            lp.set_color(rl_unit_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }

        // 4. Bottom-Right Tire (Rear-Right)
        let rr_is_low = TirePressures::is_low(tires.rr);
        let rr_color = if rr_is_low {
            alert_red
        } else {
            colors::text_primary()
        };
        let rr_unit_color = if rr_is_low {
            alert_red
        } else {
            colors::accent_cyan()
        };
        let rr_val = format!("{:.1}", tires.rr);
        let rr_val_w = fonts.measure_width(&fonts.bold, &rr_val, 28.0);
        let rr_unit_w = fonts.measure_width(&fonts.regular, "bar", 20.0);
        fonts.draw_text(
            pixmap,
            &fonts.bold,
            &rr_val,
            380.0 - rr_val_w,
            335.0,
            28.0,
            rr_color,
        );
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            "bar",
            380.0 - rr_unit_w,
            360.0,
            20.0,
            rr_unit_color,
        );

        let mut pb_rr = PathBuilder::new();
        pb_rr.move_to(335.0, 345.0);
        pb_rr.line_to(307.0, 345.0);
        pb_rr.line_to(fr_x + WHEEL_WIDTH + 3.0, rear_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_rr.finish() {
            let mut lp = Paint::default();
            lp.set_color(rr_unit_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }
    }
}
