// TPMS overlay: tire pressure & temperature readouts with horizontal separator and leader lines to wheels

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
            width: 1.6,
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: None,
        };

        let alert_red = colors::alert_red();

        // 1. Top-Left Tire (Front-Left)
        let fl_disconnected = tires.is_disconnected(tires.fl);
        let fl_is_low = tires.is_low(tires.fl);
        let fl_is_hot = tires.is_high_temp(tires.fl_temp);
        let fl_line_color = if fl_disconnected || fl_is_low || fl_is_hot {
            alert_red
        } else {
            colors::accent_cyan()
        };

        if fl_disconnected {
            // Disconnected Icon + Red ERROR text
            Self::draw_disconnected_icon(pixmap, 68.0, 118.5, alert_red);
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                "ERROR",
                81.0,
                126.0,
                19.0,
                alert_red,
            );
        } else {
            let fl_p_color = if fl_is_low {
                alert_red
            } else {
                colors::text_primary()
            };
            let fl_u_color = if fl_is_low {
                alert_red
            } else {
                colors::text_muted()
            };
            let fl_p_val = format!("{:.1}", tires.fl);
            let fl_p_w = fonts.measure_width(&fonts.bold, &fl_p_val, 22.0);

            // Pressure (Top)
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                &fl_p_val,
                60.0,
                126.0,
                22.0,
                fl_p_color,
            );
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                " bar",
                60.0 + fl_p_w,
                126.0,
                13.0,
                fl_u_color,
            );
        }

        // Horizontal Separator & Leader Line to Front-Left Wheel
        let mut pb_fl = PathBuilder::new();
        pb_fl.move_to(60.0, 135.0);
        pb_fl.line_to(130.0, 135.0);
        pb_fl.line_to(142.0, 135.0);
        pb_fl.line_to(fl_x - 3.0, front_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_fl.finish() {
            let mut lp = Paint::default();
            lp.set_color(fl_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }

        // Temperature (Bottom)
        if fl_disconnected {
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                "--°C",
                60.0,
                153.0,
                15.0,
                colors::text_muted(),
            );
        } else {
            let fl_t_color = if fl_is_hot {
                alert_red
            } else {
                colors::text_muted()
            };
            let fl_t_val = format!("{:.0}°C", tires.fl_temp);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                &fl_t_val,
                60.0,
                153.0,
                15.0,
                fl_t_color,
            );
        }

        // 2. Top-Right Tire (Front-Right)
        let fr_disconnected = tires.is_disconnected(tires.fr);
        let fr_is_low = tires.is_low(tires.fr);
        let fr_is_hot = tires.is_high_temp(tires.fr_temp);
        let fr_line_color = if fr_disconnected || fr_is_low || fr_is_hot {
            alert_red
        } else {
            colors::accent_cyan()
        };
        let fr_block_right = 380.0;

        if fr_disconnected {
            let err_w = fonts.measure_width(&fonts.bold, "ERROR", 19.0);
            let icon_box_w = 16.0;
            let gap = 7.0;
            let total_w = icon_box_w + gap + err_w;
            let start_x = fr_block_right - total_w;

            Self::draw_disconnected_icon(pixmap, start_x + (icon_box_w * 0.5), 118.5, alert_red);
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                "ERROR",
                start_x + icon_box_w + gap,
                126.0,
                19.0,
                alert_red,
            );
        } else {
            let fr_p_color = if fr_is_low {
                alert_red
            } else {
                colors::text_primary()
            };
            let fr_u_color = if fr_is_low {
                alert_red
            } else {
                colors::text_muted()
            };
            let fr_p_val = format!("{:.1}", tires.fr);
            let fr_p_w = fonts.measure_width(&fonts.bold, &fr_p_val, 22.0);
            let fr_u_w = fonts.measure_width(&fonts.regular, " bar", 13.0);
            let fr_p_start = fr_block_right - (fr_p_w + fr_u_w);

            fonts.draw_text(
                pixmap,
                &fonts.bold,
                &fr_p_val,
                fr_p_start,
                126.0,
                22.0,
                fr_p_color,
            );
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                " bar",
                fr_p_start + fr_p_w,
                126.0,
                13.0,
                fr_u_color,
            );
        }

        // Horizontal Separator & Leader Line to Front-Right Wheel
        let mut pb_fr = PathBuilder::new();
        pb_fr.move_to(fr_block_right, 135.0);
        pb_fr.line_to(310.0, 135.0);
        pb_fr.line_to(298.0, 135.0);
        pb_fr.line_to(fr_x + WHEEL_WIDTH + 3.0, front_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_fr.finish() {
            let mut lp = Paint::default();
            lp.set_color(fr_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }

        // Temperature (Bottom)
        if fr_disconnected {
            let t_w = fonts.measure_width(&fonts.regular, "--°C", 15.0);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                "--°C",
                fr_block_right - t_w,
                153.0,
                15.0,
                colors::text_muted(),
            );
        } else {
            let fr_t_color = if fr_is_hot {
                alert_red
            } else {
                colors::text_muted()
            };
            let fr_t_val = format!("{:.0}°C", tires.fr_temp);
            let fr_t_w = fonts.measure_width(&fonts.regular, &fr_t_val, 15.0);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                &fr_t_val,
                fr_block_right - fr_t_w,
                153.0,
                15.0,
                fr_t_color,
            );
        }

        // 3. Bottom-Left Tire (Rear-Left)
        let rl_disconnected = tires.is_disconnected(tires.rl);
        let rl_is_low = tires.is_low(tires.rl);
        let rl_is_hot = tires.is_high_temp(tires.rl_temp);
        let rl_line_color = if rl_disconnected || rl_is_low || rl_is_hot {
            alert_red
        } else {
            colors::accent_cyan()
        };

        if rl_disconnected {
            Self::draw_disconnected_icon(pixmap, 68.0, 320.5, alert_red);
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                "ERROR",
                81.0,
                126.0 + 202.0, // 328.0
                19.0,
                alert_red,
            );
        } else {
            let rl_p_color = if rl_is_low {
                alert_red
            } else {
                colors::text_primary()
            };
            let rl_u_color = if rl_is_low {
                alert_red
            } else {
                colors::text_muted()
            };
            let rl_p_val = format!("{:.1}", tires.rl);
            let rl_p_w = fonts.measure_width(&fonts.bold, &rl_p_val, 22.0);

            // Pressure (Top)
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                &rl_p_val,
                60.0,
                328.0,
                22.0,
                rl_p_color,
            );
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                " bar",
                60.0 + rl_p_w,
                328.0,
                13.0,
                rl_u_color,
            );
        }

        // Horizontal Separator & Leader Line to Rear-Left Wheel
        let mut pb_rl = PathBuilder::new();
        pb_rl.move_to(60.0, 337.0);
        pb_rl.line_to(130.0, 337.0);
        pb_rl.line_to(142.0, 337.0);
        pb_rl.line_to(fl_x - 3.0, rear_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_rl.finish() {
            let mut lp = Paint::default();
            lp.set_color(rl_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }

        // Temperature (Bottom)
        if rl_disconnected {
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                "--°C",
                60.0,
                355.0,
                15.0,
                colors::text_muted(),
            );
        } else {
            let rl_t_color = if rl_is_hot {
                alert_red
            } else {
                colors::text_muted()
            };
            let rl_t_val = format!("{:.0}°C", tires.rl_temp);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                &rl_t_val,
                60.0,
                355.0,
                15.0,
                rl_t_color,
            );
        }

        // 4. Bottom-Right Tire (Rear-Right)
        let rr_disconnected = tires.is_disconnected(tires.rr);
        let rr_is_low = tires.is_low(tires.rr);
        let rr_is_hot = tires.is_high_temp(tires.rr_temp);
        let rr_line_color = if rr_disconnected || rr_is_low || rr_is_hot {
            alert_red
        } else {
            colors::accent_cyan()
        };
        let rr_block_right = 380.0;

        if rr_disconnected {
            let err_w = fonts.measure_width(&fonts.bold, "ERROR", 19.0);
            let icon_box_w = 16.0;
            let gap = 7.0;
            let total_w = icon_box_w + gap + err_w;
            let start_x = rr_block_right - total_w;

            Self::draw_disconnected_icon(pixmap, start_x + (icon_box_w * 0.5), 320.5, alert_red);
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                "ERROR",
                start_x + icon_box_w + gap,
                328.0,
                19.0,
                alert_red,
            );
        } else {
            let rr_p_color = if rr_is_low {
                alert_red
            } else {
                colors::text_primary()
            };
            let rr_u_color = if rr_is_low {
                alert_red
            } else {
                colors::text_muted()
            };
            let rr_p_val = format!("{:.1}", tires.rr);
            let rr_p_w = fonts.measure_width(&fonts.bold, &rr_p_val, 22.0);
            let rr_u_w = fonts.measure_width(&fonts.regular, " bar", 13.0);
            let rr_p_start = rr_block_right - (rr_p_w + rr_u_w);

            fonts.draw_text(
                pixmap,
                &fonts.bold,
                &rr_p_val,
                rr_p_start,
                328.0,
                22.0,
                rr_p_color,
            );
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                " bar",
                rr_p_start + rr_p_w,
                328.0,
                13.0,
                rr_u_color,
            );
        }

        // Horizontal Separator & Leader Line to Rear-Right Wheel
        let mut pb_rr = PathBuilder::new();
        pb_rr.move_to(rr_block_right, 337.0);
        pb_rr.line_to(310.0, 337.0);
        pb_rr.line_to(298.0, 337.0);
        pb_rr.line_to(fr_x + WHEEL_WIDTH + 3.0, rear_y + (WHEEL_HEIGHT / 2.0));
        if let Some(path) = pb_rr.finish() {
            let mut lp = Paint::default();
            lp.set_color(rr_line_color);
            lp.anti_alias = true;
            pixmap.stroke_path(&path, &lp, &stroke_config, Transform::identity(), None);
        }

        // Temperature (Bottom)
        if rr_disconnected {
            let t_w = fonts.measure_width(&fonts.regular, "--°C", 15.0);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                "--°C",
                rr_block_right - t_w,
                355.0,
                15.0,
                colors::text_muted(),
            );
        } else {
            let rr_t_color = if rr_is_hot {
                alert_red
            } else {
                colors::text_muted()
            };
            let rr_t_val = format!("{:.0}°C", tires.rr_temp);
            let rr_t_w = fonts.measure_width(&fonts.regular, &rr_t_val, 15.0);
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                &rr_t_val,
                rr_block_right - rr_t_w,
                355.0,
                15.0,
                rr_t_color,
            );
        }
    }

    /// Minimalist TPMS Disconnected / Error Icon (Wireless Beacon with Slash)
    fn draw_disconnected_icon(pixmap: &mut Pixmap, cx: f32, cy: f32, color: tiny_skia::Color) {
        let mut pb = PathBuilder::new();
        // 1. Concentric radio wave arcs (TPMS BLE signal)
        // Inner arc
        pb.move_to(cx - 3.8, cy + 0.8);
        pb.quad_to(cx, cy - 2.5, cx + 3.8, cy + 0.8);
        // Outer arc
        pb.move_to(cx - 6.8, cy - 3.0);
        pb.quad_to(cx, cy - 7.5, cx + 6.8, cy - 3.0);
        // Base beacon dot
        pb.move_to(cx - 1.2, cy + 4.5);
        pb.line_to(cx + 1.2, cy + 4.5);

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.5,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }

        // 2. Bold diagonal slash cutting through (Disconnected indicator)
        let mut pb_slash = PathBuilder::new();
        pb_slash.move_to(cx - 7.5, cy + 6.0);
        pb_slash.line_to(cx + 7.5, cy - 7.5);
        if let Some(path) = pb_slash.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.8,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}
