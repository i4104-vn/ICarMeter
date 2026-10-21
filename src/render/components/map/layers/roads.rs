// MapLibre Road Network Layer
// Renders secondary streets, cross avenues, and primary navigation route with glow casing

use crate::render::components::map::style::MapLibreStyle;
use tiny_skia::{LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct RoadLayer;

impl RoadLayer {
    /// Renders the complete road network: secondary street grid and primary navigation route
    pub fn draw(pixmap: &mut Pixmap, style: &MapLibreStyle, transform: Transform, center_y: f32) {
        Self::draw_secondary_streets(pixmap, style, transform, center_y);
        Self::draw_primary_navigation_route(pixmap, style, transform);
    }

    /// Secondary streets and cross avenues
    fn draw_secondary_streets(
        pixmap: &mut Pixmap,
        style: &MapLibreStyle,
        transform: Transform,
        center_y: f32,
    ) {
        let mut pb = PathBuilder::new();

        // Horizontal cross streets
        let start_idx = ((center_y - 600.0) / 140.0).floor() as i32;
        for i in start_idx..=(start_idx + 10) {
            let y = i as f32 * 140.0;
            pb.move_to(-350.0, y);
            pb.line_to(350.0, y);

            pb.move_to(-350.0, y + 60.0);
            pb.line_to(350.0, y + 80.0);
        }

        // Vertical avenues
        for x in [-260.0, -130.0, 50.0, 160.0, 270.0] {
            pb.move_to(x, center_y - 600.0);
            pb.line_to(x, center_y + 600.0);
        }

        if let Some(path) = pb.finish() {
            let mut road_paint = Paint::default();
            road_paint.set_color(style.road_secondary.color);
            let stroke = Stroke {
                width: style.road_secondary.width,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &road_paint, &stroke, transform, None);
        }
    }

    /// Primary arterial navigation route with glowing casing and sharp center core
    fn draw_primary_navigation_route(
        pixmap: &mut Pixmap,
        style: &MapLibreStyle,
        transform: Transform,
    ) {
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, 1600.0);
        pb.cubic_to(40.0, 1000.0, -35.0, 400.0, 0.0, -200.0);
        pb.cubic_to(45.0, -700.0, -25.0, -1100.0, 15.0, -1600.0);

        if let Some(path) = pb.finish() {
            // 1. Outer Glow Casing (MapLibre style cobalt halo)
            let mut glow_paint = Paint::default();
            glow_paint.set_color(style.route_glow.color);
            let glow_stroke = Stroke {
                width: style.route_glow.width,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &glow_paint, &glow_stroke, transform, None);

            // 2. Navigation Route Line (Vibrant electric cyan)
            let mut route_paint = Paint::default();
            route_paint.set_color(style.route_line.color);
            let route_stroke = Stroke {
                width: style.route_line.width,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &route_paint, &route_stroke, transform, None);

            // 3. Crisp White Core
            let mut core_paint = Paint::default();
            core_paint.set_color(style.route_core.color);
            let core_stroke = Stroke {
                width: style.route_core.width,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &core_paint, &core_stroke, transform, None);
        }
    }
}
