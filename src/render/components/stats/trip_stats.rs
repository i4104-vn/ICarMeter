// Trip and driving statistics display

use crate::configs::colors;
use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::{LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub struct TripStats;

impl TripStats {
    pub fn draw(pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState) {
        let label_x = 905.0;
        let value_right_x = 1210.0;
        let divider_x1 = 905.0;
        let divider_x2 = 1210.0;

        let rows = [
            ("Driving Time", state.formatted_driving_time()),
            ("Distance (Session)", state.formatted_distance()),
            ("Max Speed", state.formatted_max_speed()),
            ("Avg Speed", state.formatted_avg_speed()),
        ];

        let baseline_y_coords = [142.0, 207.0, 272.0, 337.0];
        let divider_y_coords = [172.0, 237.0, 302.0];

        let mut div_paint = Paint::default();
        div_paint.set_color(colors::divider());
        div_paint.anti_alias = true;

        let div_stroke = Stroke {
            width: 1.2,
            miter_limit: 4.0,
            line_cap: LineCap::Round,
            dash: None,
            ..Default::default()
        };

        // Draw the 3 horizontal divider lines
        for &div_y in &divider_y_coords {
            let mut pb = PathBuilder::new();
            pb.move_to(divider_x1, div_y);
            pb.line_to(divider_x2, div_y);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &div_paint, &div_stroke, Transform::identity(), None);
            }
        }

        // Draw labels and values
        for (i, (label, value)) in rows.iter().enumerate() {
            let baseline_y = baseline_y_coords[i];

            // Label (Left-aligned, muted blue/gray)
            fonts.draw_text(
                pixmap,
                &fonts.regular,
                label,
                label_x,
                baseline_y,
                20.0,
                colors::text_muted(),
            );

            // Value (Right-aligned, bold white)
            let val_width = fonts.measure_width(&fonts.bold, value, 23.0);
            let val_x = value_right_x - val_width;
            fonts.draw_text(
                pixmap,
                &fonts.bold,
                value,
                val_x,
                baseline_y,
                23.0,
                colors::text_primary(),
            );
        }
    }
}
