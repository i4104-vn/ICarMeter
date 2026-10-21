// MapLibre Building / Landuse Layer
// Renders urban city block footprints and structural building outlines

use crate::render::components::map::style::MapLibreStyle;
use tiny_skia::{Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

pub struct BuildingLayer;

impl BuildingLayer {
    /// Renders building footprint rectangles along the transit corridor
    pub fn draw(pixmap: &mut Pixmap, style: &MapLibreStyle, transform: Transform, center_y: f32) {
        let mut block_paint = Paint::default();
        block_paint.set_color(style.building_fill.fill_color);

        let mut outline_paint = Paint::default();
        if let Some(outline_color) = style.building_fill.outline_color {
            outline_paint.set_color(outline_color);
        }

        let stroke = Stroke {
            width: style.building_fill.outline_width,
            ..Default::default()
        };

        let start_idx = ((center_y - 500.0) / 140.0).floor() as i32;
        for i in start_idx..=(start_idx + 8) {
            let by = i as f32 * 140.0;
            for bx in [-240.0, -110.0, 70.0, 180.0] {
                if let Some(rect) = Rect::from_xywh(bx, by + 15.0, 75.0, 95.0) {
                    pixmap.fill_rect(rect, &block_paint, transform, None);
                    if style.building_fill.outline_color.is_some() {
                        let r_path = PathBuilder::from_rect(rect);
                        pixmap.stroke_path(&r_path, &outline_paint, &stroke, transform, None);
                    }
                }
            }
        }
    }
}
