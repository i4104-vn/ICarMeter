// Container card frame, background styling, and section dividers

use crate::configs::colors;
use crate::render::core::create_rounded_rect;
use tiny_skia::{FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct CardContainer;

impl CardContainer {
    pub fn draw(pixmap: &mut Pixmap) {
        // 1. Fill outer background
        pixmap.fill(colors::outer_bg());

        // 2. Draw Main Card Container
        let card_x = 35.0;
        let card_y = 30.0;
        let card_w = 1210.0;
        let card_h = 420.0;
        let card_radius = 24.0;

        let card_path = create_rounded_rect(card_x, card_y, card_w, card_h, card_radius);

        // Fill card background
        let mut card_bg_paint = Paint::default();
        card_bg_paint.set_color(colors::card_bg());
        card_bg_paint.anti_alias = true;
        pixmap.fill_path(
            &card_path,
            &card_bg_paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );

        // Card outer border glow
        let mut glow_paint = Paint::default();
        glow_paint.set_color(colors::card_border_glow());
        glow_paint.anti_alias = true;
        let glow_stroke = Stroke {
            width: 4.5,
            line_cap: LineCap::Round,
            ..Default::default()
        };
        pixmap.stroke_path(
            &card_path,
            &glow_paint,
            &glow_stroke,
            Transform::identity(),
            None,
        );

        // Card border stroke
        let mut border_paint = Paint::default();
        border_paint.set_color(colors::card_border());
        border_paint.anti_alias = true;
        let border_stroke = Stroke {
            width: 1.8,
            line_cap: LineCap::Round,
            ..Default::default()
        };
        pixmap.stroke_path(
            &card_path,
            &border_paint,
            &border_stroke,
            Transform::identity(),
            None,
        );

        // 3. Draw Vertical Section Dividing Lines
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
        pb_div1.move_to(410.0, 55.0);
        pb_div1.line_to(410.0, 425.0);
        if let Some(path) = pb_div1.finish() {
            pixmap.stroke_path(&path, &div_paint, &div_stroke, Transform::identity(), None);
        }

        // Divider 2 (Center / Right)
        let mut pb_div2 = PathBuilder::new();
        pb_div2.move_to(870.0, 55.0);
        pb_div2.line_to(870.0, 425.0);
        if let Some(path) = pb_div2.finish() {
            pixmap.stroke_path(&path, &div_paint, &div_stroke, Transform::identity(), None);
        }
    }
}
