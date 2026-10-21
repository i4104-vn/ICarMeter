// MapLibre Tactical Grid Layer
// Renders coordinate reference grid lines across the map viewport

use crate::render::components::map::style::MapLibreStyle;
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct GridLayer;

impl GridLayer {
    /// Draws the coordinate reference grid transformed by the MapLibre camera matrix
    pub fn draw(pixmap: &mut Pixmap, style: &MapLibreStyle, transform: Transform, center_y: f32) {
        let mut grid_paint = Paint::default();
        grid_paint.set_color(style.grid_color);
        let grid_stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        let start_y = ((center_y - 600.0) / 90.0).floor() as i32 * 90;
        let end_y = start_y + 1200;

        let mut pb = PathBuilder::new();
        // Longitude grid lines
        for gx in (-4..=4).map(|i| i as f32 * 90.0) {
            pb.move_to(gx, start_y as f32);
            pb.line_to(gx, end_y as f32);
        }
        // Latitude grid lines
        for gy in (start_y..=end_y).step_by(90) {
            pb.move_to(-360.0, gy as f32);
            pb.line_to(360.0, gy as f32);
        }

        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &grid_paint, &grid_stroke, transform, None);
        }
    }
}
