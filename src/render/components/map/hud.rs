// Tactical HUD Overlays for MapLibre 2D Navigation
// Displays Compass Card, 3D GPS Fix Badge, Coordinates Pill, and Distance Scale Bar.

use super::style::MapLibreStyle;
use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

pub struct HudOverlay;

impl HudOverlay {
    /// Renders all HUD interface elements over the map viewport
    pub fn draw(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        style: &MapLibreStyle,
        w: f32,
        h: f32,
        heading_deg: f32,
        state: &VehicleState,
    ) {
        Self::draw_compass_card(pixmap, fonts, style, 12.0, 12.0, heading_deg);
        Self::draw_gps_fix_badge(pixmap, fonts, style, w, 12.0, state.gps_connected);
        Self::draw_coordinates_pill(pixmap, fonts, style, 12.0, h - 30.0, state);
        Self::draw_scale_bar(pixmap, fonts, style, w - 56.0 - 12.0, h - 30.0);
    }

    /// Top-Left Compass Card displaying Cardinal direction and degree
    fn draw_compass_card(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        style: &MapLibreStyle,
        x: f32,
        y: f32,
        heading_deg: f32,
    ) {
        Self::draw_card_background(pixmap, style, x, y, 76.0, 30.0);

        let cardinal = match heading_deg {
            d if (337.5..=360.0).contains(&d) || (0.0..22.5).contains(&d) => "N",
            d if (22.5..67.5).contains(&d) => "NE",
            d if (67.5..112.5).contains(&d) => "E",
            d if (112.5..157.5).contains(&d) => "SE",
            d if (157.5..202.5).contains(&d) => "S",
            d if (202.5..247.5).contains(&d) => "SW",
            d if (247.5..292.5).contains(&d) => "W",
            _ => "NW",
        };

        let heading_text = format!("{:>2} {:03.0}", cardinal, heading_deg);
        fonts.draw_text(
            pixmap,
            &fonts.bold,
            &heading_text,
            x + 8.0,
            y + 19.0,
            12.0,
            style.hud_text_primary,
        );

        // Vector degree ring (avoiding missing font glyphs)
        let deg_cx = x + 64.0;
        let deg_cy = y + 11.0;
        let mut pb_deg = PathBuilder::new();
        pb_deg.push_circle(deg_cx, deg_cy, 2.0);
        if let Some(path) = pb_deg.finish() {
            let mut deg_paint = Paint::default();
            deg_paint.set_color(style.hud_text_primary);
            let stroke = Stroke {
                width: 1.0,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &deg_paint, &stroke, Transform::identity(), None);
        }
    }

    /// Top-Right GPS Fix Status Badge
    fn draw_gps_fix_badge(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        style: &MapLibreStyle,
        viewport_w: f32,
        y: f32,
        gps_connected: bool,
    ) {
        let badge_w = 78.0;
        let x = viewport_w - badge_w - 12.0;
        Self::draw_card_background(pixmap, style, x, y, badge_w, 26.0);

        let (dot_color, label) = match gps_connected {
            true => (style.hud_fix_dot, "3D FIX"),
            false => (style.hud_text_muted, "DEMO"),
        };

        // Fix status circle dot
        let mut pb_dot = PathBuilder::new();
        pb_dot.push_circle(x + 12.0, y + 13.0, 3.0);
        if let Some(path) = pb_dot.finish() {
            let mut dot_paint = Paint::default();
            dot_paint.set_color(dot_color);
            pixmap.fill_path(&path, &dot_paint, FillRule::Winding, Transform::identity(), None);
        }

        fonts.draw_text(pixmap, &fonts.bold, label, x + 22.0, y + 17.0, 10.0, dot_color);
    }

    /// Bottom-Left GPS Coordinates Pill
    fn draw_coordinates_pill(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        style: &MapLibreStyle,
        x: f32,
        y: f32,
        state: &VehicleState,
    ) {
        Self::draw_card_background(pixmap, style, x, y, 180.0, 20.0);
        let coord_text = format!("{:.4} N  {:.4} E", state.gps_latitude, state.gps_longitude);
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            &coord_text,
            x + 8.0,
            y + 14.0,
            10.0,
            style.hud_text_muted,
        );
    }

    /// Bottom-Right Distance Scale Bar Indicator
    fn draw_scale_bar(
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        style: &MapLibreStyle,
        x: f32,
        y: f32,
    ) {
        let scale_w = 56.0;
        Self::draw_card_background(pixmap, style, x, y, scale_w, 20.0);
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            "200 m",
            x + 13.0,
            y + 14.0,
            10.0,
            style.hud_text_muted,
        );
    }

    /// Sleek semi-transparent HUD pill/card background with subtle border
    pub fn draw_card_background(
        pixmap: &mut Pixmap,
        style: &MapLibreStyle,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            let mut bg_paint = Paint::default();
            bg_paint.set_color(style.hud_bg);
            pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);

            let r_path = PathBuilder::from_rect(rect);
            let mut border_paint = Paint::default();
            border_paint.set_color(style.hud_border);
            let stroke = Stroke {
                width: 1.0,
                ..Default::default()
            };
            pixmap.stroke_path(&r_path, &border_paint, &stroke, Transform::identity(), None);
        }
    }
}
