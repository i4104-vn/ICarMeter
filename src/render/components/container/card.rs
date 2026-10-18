// Container background styling and vertical section dividers (full bleed, borderless)

use crate::configs::colors;
use tiny_skia::{LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct CardContainer;

impl CardContainer {
    pub fn draw(pixmap: &mut Pixmap) {
        // 1. Fill entire screen edge-to-edge with pure OLED black
        pixmap.fill(colors::outer_bg());

        // 2. Subtle Vertical Section Dividing Lines (Left / Center / Right)
        let mut div_paint = Paint::default();
        div_paint.set_color(colors::divider());
        div_paint.anti_alias = true;
        let div_stroke = Stroke {
            width: 1.2,
            line_cap: LineCap::Round,
            ..Default::default()
        };

        // Divider 1 (Left / Center)
        let mut pb_div1 = PathBuilder::new();
        pb_div1.move_to(410.0, 35.0);
        pb_div1.line_to(410.0, 425.0);
        if let Some(path) = pb_div1.finish() {
            pixmap.stroke_path(&path, &div_paint, &div_stroke, Transform::identity(), None);
        }

        // Divider 2 (Center / Right)
        let mut pb_div2 = PathBuilder::new();
        pb_div2.move_to(870.0, 35.0);
        pb_div2.line_to(870.0, 425.0);
        if let Some(path) = pb_div2.finish() {
            pixmap.stroke_path(&path, &div_paint, &div_stroke, Transform::identity(), None);
        }
    }
}
