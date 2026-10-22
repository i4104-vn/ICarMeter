use super::sprite_renderer::EXPRESSION_DURATION;
use crate::models::emotion::expression::EmoExpression;
use std::time::{SystemTime, UNIX_EPOCH};

const RANDOM_COUNT: usize = EmoExpression::RANDOM_MOODS.len();

#[derive(Clone, Copy, PartialEq, Eq)]
enum SleepCyclePhase {
    Random,
    Sleepy,
    Sleeping,
}

pub struct EmoAnimator {
    pub active_expression: Option<EmoExpression>,
    phase: SleepCyclePhase,
    mood_timer: f32,
    mood_duration: f32,
    last_mood_idx: usize,
    rng_state: u64,
    mood_order: [usize; RANDOM_COUNT],
    mood_cursor: usize,
    random_emotions_target: u8,
    random_emotions_shown: u8,
    was_moving: bool,
}

impl EmoAnimator {
    pub fn new() -> Self {
        let time_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(1);
        let mut initial_order = [0usize; RANDOM_COUNT];
        for (i, item) in initial_order.iter_mut().enumerate() {
            *item = i;
        }

        let mut animator = Self {
            active_expression: None,
            phase: SleepCyclePhase::Random,
            mood_timer: 0.0,
            mood_duration: EXPRESSION_DURATION,
            last_mood_idx: usize::MAX,
            rng_state: (time_seed ^ (std::process::id() as u64).rotate_left(23)).max(1),
            mood_order: initial_order,
            mood_cursor: RANDOM_COUNT,
            random_emotions_target: 10,
            random_emotions_shown: 0,
            was_moving: false,
        };
        animator.start_random_cycle();
        animator
    }

    fn rand_f32(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state & 0x00FF_FFFF) as f32 / 16_777_216.0
    }

    pub fn update(&mut self, dt: f32, idle_seconds: f32, forced_expr: Option<EmoExpression>) {
        if let Some(forced) = forced_expr {
            self.active_expression = Some(forced);
            return;
        }

        if idle_seconds <= 0.0 {
            if !self.was_moving {
                self.was_moving = true;
                self.start_random_cycle();
            }
            return;
        }
        self.was_moving = false;

        self.mood_timer += dt.max(0.0);
        if self.mood_timer < self.mood_duration {
            return;
        }
        self.mood_timer = 0.0;

        match self.phase {
            SleepCyclePhase::Random
                if self.random_emotions_shown >= self.random_emotions_target =>
            {
                self.phase = SleepCyclePhase::Sleepy;
                self.active_expression = Some(EmoExpression::Sleepy);
            }
            SleepCyclePhase::Random => self.apply_random_mood(),
            SleepCyclePhase::Sleepy => {
                self.phase = SleepCyclePhase::Sleeping;
                self.active_expression = Some(EmoExpression::Sleeping);
            }
            SleepCyclePhase::Sleeping => {
                self.start_random_cycle();
            }
        }
    }

    fn start_random_cycle(&mut self) {
        self.phase = SleepCyclePhase::Random;
        self.mood_timer = 0.0;
        self.mood_duration = EXPRESSION_DURATION;
        self.mood_cursor = RANDOM_COUNT;
        self.random_emotions_shown = 0;
        self.random_emotions_target = 8 + (self.rand_f32() * 6.0) as u8;
        self.apply_random_mood();
    }

    fn apply_random_mood(&mut self) {
        // Shuffle the available emotions to continuously cycle through without direct repeats.
        if self.mood_cursor >= self.mood_order.len() {
            for i in (1..self.mood_order.len()).rev() {
                let j = (self.rand_f32() * (i + 1) as f32) as usize;
                self.mood_order.swap(i, j);
            }
            if self.mood_order[0] == self.last_mood_idx {
                self.mood_order.swap(0, 1);
            }
            self.mood_cursor = 0;
        }

        let idx = self.mood_order[self.mood_cursor];
        self.mood_cursor += 1;
        self.last_mood_idx = idx;
        self.random_emotions_shown += 1;
        self.active_expression = Some(EmoExpression::RANDOM_MOODS[idx]);
    }
}
