// Randomly cycles through the individual expression SVGs and idle sleep states.

use super::sprite_renderer::{EXPRESSION_COUNT, EXPRESSION_DURATION};
use super::types::EmoExpression;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct EmoAnimator {
    pub active_expression: Option<EmoExpression>,
    mood_timer: f32,
    mood_duration: f32,
    last_mood_idx: usize,
    rng_state: u64,
    mood_order: [u8; EXPRESSION_COUNT],
    mood_cursor: usize,
}

impl EmoAnimator {
    pub fn new() -> Self {
        let time_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(1);
        let mut animator = Self {
            active_expression: Some(EmoExpression::Normal),
            mood_timer: 0.0,
            mood_duration: EXPRESSION_DURATION,
            last_mood_idx: usize::MAX,
            rng_state: (time_seed ^ (std::process::id() as u64).rotate_left(23)).max(1),
            mood_order: std::array::from_fn(|index| index as u8),
            mood_cursor: EXPRESSION_COUNT,
        };
        animator.apply_random_mood();
        animator
    }

    fn rand_f32(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state & 0x00FFFFFF) as f32 / 16777216.0
    }

    pub fn update(&mut self, dt: f32, idle_seconds: f32, forced_expr: Option<EmoExpression>) {
        if let Some(forced) = forced_expr {
            self.active_expression = Some(forced);
            return;
        }

        // Gradually fall asleep when the vehicle remains stopped.
        if idle_seconds >= 30.0 {
            self.active_expression = Some(EmoExpression::Sleeping);
            return;
        }

        // Change to another reaction at a calm, predictable pace.
        self.mood_timer += dt;
        if self.mood_timer >= self.mood_duration {
            self.mood_timer = 0.0;
            self.mood_duration = EXPRESSION_DURATION;
            self.apply_random_mood();
        }
    }

    fn apply_random_mood(&mut self) {
        // Shuffle a full bag so every SVG is used once before any expression repeats.
        if self.mood_cursor >= EXPRESSION_COUNT {
            for i in (1..EXPRESSION_COUNT).rev() {
                let j = (self.rand_f32() * (i + 1) as f32) as usize;
                self.mood_order.swap(i, j);
            }
            if self.mood_order[0] as usize == self.last_mood_idx {
                self.mood_order.swap(0, 1);
            }
            self.mood_cursor = 0;
        }
        let idx = self.mood_order[self.mood_cursor] as usize;
        self.mood_cursor += 1;
        self.last_mood_idx = idx;
        self.active_expression = Some(EmoExpression::Atlas(idx as u8));
    }
}
