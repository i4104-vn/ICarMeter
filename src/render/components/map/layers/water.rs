// MapLibre Water Feature Layer
// Renders river and water body polygons and shorelines

use crate::render::components::map::style::MapLibreStyle;
use tiny_skia::{FillRule, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct WaterLayer;

impl WaterLayer {
    /// Renders river and waterway polygons according to MapLibre style rules
    pub fn draw(pixmap: &mut Pixmap, style: &MapLibreStyle, transform: Transform) {
        let mut pb = PathBuilder::new();
        // Meandering river corridor
        pb.move_to(-300.0, -1500.0);
        pb.cubic_to(-210.0, -900.0, -320.0, -300.0, -220.0, 300.0);
        pb.cubic_to(-140.0, 800.0, -260.0, 1200.0, -200.0, 1800.0);
        pb.line_to(-120.0, 1800.0);
        pb.cubic_to(-180.0, 1200.0, -60.0, 800.0, -140.0, 300.0);
        pb.cubic_to(-240.0, -300.0, -130.0, -900.0, -220.0, -1500.0);
        pb.close();

        if let Some(path) = pb.finish() {
            // Fill water polygon
            let mut water_paint = Paint::default();
            water_paint.set_color(style.water_fill.fill_color);
            pixmap.fill_path(&path, &water_paint, FillRule::Winding, transform, None);

            // Water shoreline stroke
            if let Some(edge_color) = style.water_fill.outline_color {
                let mut edge_paint = Paint::default();
                edge_paint.set_color(edge_color);
                let edge_stroke = Stroke {
                    width: style.water_fill.outline_width,
                    line_join: LineJoin::Round,
                    ..Default::default()
                };
                pixmap.stroke_path(&path, &edge_paint, &edge_stroke, transform, None);
            }
        }
    }
}
