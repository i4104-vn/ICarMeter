// MapLibre Waypoints / Symbol Layer
// Renders tactical navigation point diamonds along the path

use crate::render::components::map::style::MapLibreStyle;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct WaypointLayer;

impl WaypointLayer {
    /// Renders waypoint markers along the route
    pub fn draw(pixmap: &mut Pixmap, style: &MapLibreStyle, transform: Transform, center_y: f32) {
        let start_idx = ((center_y - 600.0) / 400.0).floor() as i32;
        for i in start_idx..=(start_idx + 4) {
            let wy = i as f32 * 400.0;
            let wx = (wy * 0.003).sin() * 35.0;

            let mut pb = PathBuilder::new();
            pb.move_to(wx, wy - 6.0);
            pb.line_to(wx + 6.0, wy);
            pb.line_to(wx, wy + 6.0);
            pb.line_to(wx - 6.0, wy);
            pb.close();

            if let Some(path) = pb.finish() {
                // Diamond fill
                let mut pt_paint = Paint::default();
                pt_paint.set_color(style.waypoint_color);
                pixmap.fill_path(&path, &pt_paint, FillRule::Winding, transform, None);

                // White outline
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(Color::WHITE);
                let stroke = Stroke {
                    width: 1.2,
                    ..Default::default()
                };
                pixmap.stroke_path(&path, &stroke_paint, &stroke, transform, None);
            }
        }
    }
}
