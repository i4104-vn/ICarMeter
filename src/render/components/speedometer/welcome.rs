// Welcome sequence: Emo sleeping (0.0s - 4.0s) then waking up (4.0s - 8.0s)
// Clean presentation without extra text or HUD clutter

use super::transition::{WELCOME_SLEEP_END, WELCOME_WAKE_END};
use crate::models::emotion::expression::EmoExpression;
use crate::render::components::emo::EmoSpriteRenderer;
use crate::render::core::FontRenderer;
use tiny_skia::Pixmap;

pub struct SpeedometerWelcome;

impl SpeedometerWelcome {
    pub fn draw(
        pixmap: &mut Pixmap,
        _fonts: &FontRenderer,
        cx: f32,
        cy: f32,
        time: f32,
        alpha: f32,
    ) {
        if alpha <= 0.001 {
            return;
        }

        match time {
            time if time < WELCOME_SLEEP_END => {
                // 1. Emo sleeping peacefully (sleepy.svg, index 26)
                let mood = EmoExpression::Atlas(26);
                EmoSpriteRenderer::draw(pixmap, cx, cy, time, alpha, mood);
            }
            time if time < WELCOME_WAKE_END => {
                // 2. Emo waking up, yawning and opening eyes (ngap.svg, index 18)
                let mood = EmoExpression::Atlas(18);
                let wake_time = time - WELCOME_SLEEP_END;
                EmoSpriteRenderer::draw(pixmap, cx, cy, wake_time, alpha, mood);
            }
            _ => {}
        }
    }
}
