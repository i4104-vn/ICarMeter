// Animated expression sprites and random selection state.

pub mod animator;
mod sprite_parser;
pub mod sprite_renderer;
pub mod types;

use crate::render::core::FontRenderer;
use animator::EmoAnimator;
pub use sprite_renderer::EmoSpriteRenderer;
use tiny_skia::Pixmap;
pub use types::EmoExpression;

pub struct EmoRenderer {
    animator: EmoAnimator,
    current_expression: Option<EmoExpression>,
    current_expression_started_at: f32,
    last_idle_seconds: f32,
}

impl EmoRenderer {
    pub fn new() -> Self {
        Self {
            animator: EmoAnimator::new(),
            current_expression: None,
            current_expression_started_at: 0.0,
            last_idle_seconds: 0.0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        pixmap: &mut Pixmap,
        _fonts: &FontRenderer,
        center_x: f32,
        center_y: f32,
        dt: f32,
        time: f32,
        idle_seconds: f32,
        alpha: f32,
        forced_expr: Option<EmoExpression>,
    ) {
        if alpha <= 0.001 {
            return;
        }

        // Update autonomous random personality & idle sleep timer
        self.animator.update(dt, idle_seconds, forced_expr);

        let active = forced_expr
            .or(self.animator.active_expression)
            .unwrap_or(EmoExpression::Normal);
        let resumed_after_movement = idle_seconds > 0.0 && self.last_idle_seconds <= 0.0;

        if self.current_expression != Some(active) {
            self.current_expression = Some(active);
            self.current_expression_started_at = time;
        } else if resumed_after_movement {
            self.current_expression_started_at = time;
        }
        self.last_idle_seconds = idle_seconds;

        EmoSpriteRenderer::draw(
            pixmap,
            center_x,
            center_y,
            (time - self.current_expression_started_at).max(0.0),
            alpha,
            active,
        );
    }
}
