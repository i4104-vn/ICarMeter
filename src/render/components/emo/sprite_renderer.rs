//! Renderer for each expression's horizontal SVG animation strip.

use super::EmoExpression;
use super::sprite_parser::{EXPRESSION_SVGS, ExpressionSprite, parse_svg};
use std::sync::OnceLock;
use tiny_skia::{FillRule, Paint, Pixmap, Transform};

pub(super) use super::sprite_parser::{EXPRESSION_COUNT, EXPRESSION_DURATION};

const MAX_DISPLAY_WIDTH: f32 = 330.0;
const MAX_DISPLAY_HEIGHT: f32 = 125.0;
static EXPRESSIONS: OnceLock<Vec<ExpressionSprite>> = OnceLock::new();

pub struct EmoSpriteRenderer;

impl EmoSpriteRenderer {
    pub fn draw(pixmap: &mut Pixmap, cx: f32, cy: f32, time: f32, alpha: f32, mood: EmoExpression) {
        if alpha <= 0.001 {
            return;
        }

        let expressions =
            EXPRESSIONS.get_or_init(|| EXPRESSION_SVGS.iter().map(|svg| parse_svg(svg)).collect());
        let expression_index = expression_cell(mood).min(expressions.len() - 1);
        let sprite = &expressions[expression_index];
        let elapsed_frame = (time.max(0.0) * super::sprite_parser::FRAME_RATE).floor() as usize;
        let frame_index = if matches!(
            mood,
            EmoExpression::Sleeping | EmoExpression::Sleepy | EmoExpression::Atlas(26)
        ) {
            elapsed_frame % super::sprite_parser::FRAME_COUNT
        } else {
            elapsed_frame.min(super::sprite_parser::FRAME_COUNT - 1)
        };
        let frame = &sprite.frames[frame_index];
        let Some((left, top, right, bottom)) = sprite.bounds else {
            return;
        };
        let width = right - left;
        let height = bottom - top;
        let scale = (MAX_DISPLAY_WIDTH / width).min(MAX_DISPLAY_HEIGHT / height);
        let tx = cx - (left + frame_index as f32 * sprite.frame_width + width * 0.5) * scale;
        let ty = cy - (top + height * 0.5) * scale;
        let transform = Transform::from_row(scale, 0.0, 0.0, scale, tx, ty);

        for shape in frame {
            let mut paint = Paint {
                anti_alias: true,
                ..Paint::default()
            };
            paint.set_color_rgba8(
                (shape.color.red() * 255.0).round() as u8,
                (shape.color.green() * 255.0).round() as u8,
                (shape.color.blue() * 255.0).round() as u8,
                (alpha.clamp(0.0, 1.0) * shape.opacity * 255.0).round() as u8,
            );
            pixmap.fill_path(&shape.path, &paint, FillRule::EvenOdd, transform, None);
        }
    }
}

fn expression_cell(expression: EmoExpression) -> usize {
    match expression {
        EmoExpression::Atlas(index) => index as usize,
        EmoExpression::Normal => 27,
        EmoExpression::Relaxed => 28,
        EmoExpression::Winking => 19,
        EmoExpression::Angry => 10,
        EmoExpression::Rage => 6,
        EmoExpression::Sleepy | EmoExpression::Sleeping => 26,
        EmoExpression::AngryBlush => 7,
        EmoExpression::Crying => 3,
        EmoExpression::FlatSlit => 19,
        EmoExpression::Happy => 5,
        EmoExpression::Squint => 29,
        EmoExpression::Love => 34,
        EmoExpression::Sparkle => 24,
        EmoExpression::Surprised => 17,
        EmoExpression::Starry => 34,
        EmoExpression::Dizzy => 2,
    }
}
