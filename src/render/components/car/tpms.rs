// TPMS overlay: one renderer shared by all four tire positions.

use super::wheels::{WHEEL_HEIGHT, WHEEL_WIDTH};
use crate::configs::colors;
use crate::models::tpms::TirePressures;
use crate::render::core::{FontRenderer, create_rounded_rect};
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

const PRESSURE_FONT_SIZE: f32 = 22.0;
const UNIT_FONT_SIZE: f32 = 13.0;
const TEMPERATURE_FONT_SIZE: f32 = 16.0;
const UNIT: &str = " bar";
const DISCONNECTED_TEMPERATURE: &str = "-- \u{00b0}C";

pub struct TpmsOverlay;

#[derive(Clone, Copy)]
struct TireLayout {
    pressure: f32,
    temperature: f32,
    text_x: f32,
    pressure_y: f32,
    temperature_y: f32,
    badge_y: f32,
    line_y: f32,
    line_mid_x: f32,
    target_x: f32,
    target_y: f32,
    align_right: bool,
}

#[derive(Clone, Copy)]
struct TireColors {
    alert: Color,
    line: Color,
    normal_temperature: Color,
    disconnected_temperature: Color,
}

impl TpmsOverlay {
    pub fn draw(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        cx: f32,
        cy: f32,
        tires: &TirePressures,
    ) {
        let front_y = cy - 58.0;
        let rear_y = cy + 22.0;
        let left_target_x = cx - 44.0 - WHEEL_WIDTH - 6.0;
        let right_target_x = cx + 44.0 + WHEEL_WIDTH + 6.0;

        let layouts = [
            TireLayout {
                pressure: tires.fl,
                temperature: tires.fl_temp,
                text_x: 60.0,
                pressure_y: 126.0,
                temperature_y: 155.0,
                badge_y: 110.0,
                line_y: 135.0,
                line_mid_x: 144.0,
                target_x: left_target_x,
                target_y: front_y + WHEEL_HEIGHT / 2.0,
                align_right: false,
            },
            TireLayout {
                pressure: tires.fr,
                temperature: tires.fr_temp,
                text_x: 380.0,
                pressure_y: 126.0,
                temperature_y: 155.0,
                badge_y: 110.0,
                line_y: 135.0,
                line_mid_x: 296.0,
                target_x: right_target_x,
                target_y: front_y + WHEEL_HEIGHT / 2.0,
                align_right: true,
            },
            TireLayout {
                pressure: tires.rl,
                temperature: tires.rl_temp,
                text_x: 60.0,
                pressure_y: 328.0,
                temperature_y: 357.0,
                badge_y: 312.0,
                line_y: 337.0,
                line_mid_x: 144.0,
                target_x: left_target_x,
                target_y: rear_y + WHEEL_HEIGHT / 2.0,
                align_right: false,
            },
            TireLayout {
                pressure: tires.rr,
                temperature: tires.rr_temp,
                text_x: 380.0,
                pressure_y: 328.0,
                temperature_y: 357.0,
                badge_y: 312.0,
                line_y: 337.0,
                line_mid_x: 296.0,
                target_x: right_target_x,
                target_y: rear_y + WHEEL_HEIGHT / 2.0,
                align_right: true,
            },
        ];

        let stroke = Stroke {
            width: 1.2,
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: None,
        };
        let palette = TireColors {
            alert: Color::from_rgba8(245, 55, 55, 255),
            line: colors::accent_cyan(),
            normal_temperature: Color::from_rgba8(220, 226, 236, 255),
            disconnected_temperature: Color::from_rgba8(160, 168, 180, 220),
        };

        for layout in layouts {
            Self::draw_tire(pixmap, fonts, tires, layout, &stroke, palette);
        }
    }

    fn draw_tire(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        tires: &TirePressures,
        layout: TireLayout,
        stroke: &Stroke,
        palette: TireColors,
    ) {
        let disconnected = tires.is_disconnected(layout.pressure);
        let low_pressure = tires.is_low(layout.pressure);
        let high_temperature = tires.is_high_temp(layout.temperature);
        let line_color = if disconnected || low_pressure || high_temperature {
            palette.alert
        } else {
            palette.line
        };

        if disconnected {
            Self::draw_error_badge(
                pixmap,
                fonts,
                layout.text_x - if layout.align_right { 76.0 } else { 0.0 },
                layout.badge_y,
                76.0,
                20.0,
                palette.alert,
            );
        } else {
            Self::draw_pressure(pixmap, fonts, layout, low_pressure, palette.alert);
        }

        Self::draw_leader_line(pixmap, layout, stroke, line_color);
        Self::draw_temperature(
            pixmap,
            fonts,
            layout,
            high_temperature,
            disconnected,
            palette,
        );
    }

    fn draw_pressure(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        layout: TireLayout,
        low_pressure: bool,
        alert_color: Color,
    ) {
        let pressure = format!("{:.1}", layout.pressure);
        let pressure_width = fonts.measure_width(&fonts.bold, &pressure, PRESSURE_FONT_SIZE);
        let unit_width = fonts.measure_width(&fonts.regular, UNIT, UNIT_FONT_SIZE);
        let pressure_x = if layout.align_right {
            layout.text_x - pressure_width - unit_width
        } else {
            layout.text_x
        };
        let pressure_color = if low_pressure {
            alert_color
        } else {
            colors::text_primary()
        };
        let unit_color = if low_pressure {
            alert_color
        } else {
            colors::text_muted()
        };

        fonts.draw_text(
            pixmap,
            &fonts.bold,
            &pressure,
            pressure_x,
            layout.pressure_y,
            PRESSURE_FONT_SIZE,
            pressure_color,
        );
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            UNIT,
            pressure_x + pressure_width,
            layout.pressure_y,
            UNIT_FONT_SIZE,
            unit_color,
        );
    }

    fn draw_temperature(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        layout: TireLayout,
        high_temperature: bool,
        disconnected: bool,
        palette: TireColors,
    ) {
        let value = if disconnected {
            DISCONNECTED_TEMPERATURE.to_string()
        } else {
            format!("{:.0}\u{00b0}C", layout.temperature)
        };
        let color = match (disconnected, high_temperature) {
            (true, _) => palette.disconnected_temperature,
            (false, true) => palette.alert,
            (false, false) => palette.normal_temperature,
        };
        let x = if layout.align_right {
            layout.text_x - fonts.measure_width(&fonts.bold, &value, TEMPERATURE_FONT_SIZE)
        } else {
            layout.text_x
        };

        fonts.draw_text(
            pixmap,
            &fonts.bold,
            &value,
            x,
            layout.temperature_y,
            TEMPERATURE_FONT_SIZE,
            color,
        );
    }

    fn draw_leader_line(pixmap: &mut Pixmap, layout: TireLayout, stroke: &Stroke, color: Color) {
        let mut path = PathBuilder::new();
        path.move_to(layout.text_x, layout.line_y);
        path.line_to(layout.line_mid_x, layout.line_y);
        path.line_to(layout.target_x, layout.target_y);
        if let Some(path) = path.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.stroke_path(&path, &paint, stroke, Transform::identity(), None);
        }
        Self::draw_circle(pixmap, layout.target_x, layout.target_y, 2.2, color);
    }

    fn draw_error_badge(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
    ) {
        let badge = create_rounded_rect(x, y, w, h, 4.0);
        let mut background = Paint::default();
        background.set_color(Color::from_rgba8(235, 35, 35, 26));
        background.anti_alias = true;
        pixmap.fill_path(
            &badge,
            &background,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );

        let mut border = Paint::default();
        border.set_color(Color::from_rgba8(235, 55, 55, 140));
        border.anti_alias = true;
        pixmap.stroke_path(
            &badge,
            &border,
            &Stroke {
                width: 1.0,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );

        let icon_x = x + 13.0;
        let icon_y = y + h / 2.0;
        let mut triangle = PathBuilder::new();
        triangle.move_to(icon_x, icon_y - 4.5);
        triangle.line_to(icon_x + 4.8, icon_y + 4.0);
        triangle.line_to(icon_x - 4.8, icon_y + 4.0);
        triangle.close();
        if let Some(path) = triangle.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.stroke_path(
                &path,
                &paint,
                &Stroke {
                    width: 1.2,
                    line_cap: LineCap::Round,
                    line_join: LineJoin::Round,
                    ..Default::default()
                },
                Transform::identity(),
                None,
            );
        }

        let mut mark = PathBuilder::new();
        mark.move_to(icon_x, icon_y - 2.0);
        mark.line_to(icon_x, icon_y + 0.8);
        if let Some(path) = mark.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.stroke_path(
                &path,
                &paint,
                &Stroke {
                    width: 1.2,
                    line_cap: LineCap::Round,
                    ..Default::default()
                },
                Transform::identity(),
                None,
            );
        }
        Self::draw_circle(pixmap, icon_x, icon_y + 2.7, 0.7, color);
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

    fn draw_circle(pixmap: &mut Pixmap, cx: f32, cy: f32, radius: f32, color: Color) {
        let mut path = PathBuilder::new();
        path.push_circle(cx, cy, radius);
        if let Some(path) = path.finish() {
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
