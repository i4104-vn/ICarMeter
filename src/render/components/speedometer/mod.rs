// Speedometer & Emo Companion Center Panel Facade

pub mod brackets;
pub mod digits;
pub mod particles;
pub mod tier;
pub mod transition;

use super::emo::{EmoExpression, EmoRenderer};
use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use crate::services::ParticleEngine;
use brackets::SpeedometerBrackets;
use digits::SpeedometerDigits;
use particles::ParticleRenderer;
use tier::get_tier_colors;
use tiny_skia::Pixmap;
use transition::{CenterModeState, CenterTransitionManager};

pub struct SpeedometerPanel {
    engine: ParticleEngine,
    emo: EmoRenderer,
    transitions: CenterTransitionManager,
}

impl SpeedometerPanel {
    pub fn new() -> Self {
        Self {
            engine: ParticleEngine::new(),
            emo: EmoRenderer::new(),
            transitions: CenterTransitionManager::new(),
        }
    }

    /// Fast-forward immediately to Emo mode (e.g. for snapshot testing)
    pub fn set_immediate_emo(&mut self) {
        self.transitions.state = CenterModeState::EmoActive;
        self.transitions.idle_zero_timer = 3.5;
    }

    pub fn draw(
        &mut self,
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        state: &VehicleState,
        dt: f32,
        forced_emo: Option<EmoExpression>,
    ) {
        let center_x = 640.0;
        let center_y = 240.0;
        let speed = state.current_speed;

        // If forced expression provided, lock immediately to Emo mode
        if forced_emo.is_some() {
            self.transitions.state = CenterModeState::EmoActive;
        }

        // 1. Update center state machine
        self.transitions.update(speed, dt);
        let (emo_a, brk_fly, brk_a, digit_a, particle_a) = self.transitions.visual_parameters();

        // 2. Draw Emo digital companion eyes if active/fading
        if emo_a > 0.001 {
            self.emo.draw(
                pixmap,
                fonts,
                center_x,
                center_y,
                dt,
                self.transitions.elapsed_time,
                self.transitions.idle_zero_timer,
                emo_a,
                forced_emo,
            );
        }

        // 3. Draw Speedometer elements if active/fading
        let (core_color, glow_color) = get_tier_colors(speed);

        // Suction particles
        if particle_a > 0.001 {
            self.engine.update(speed, dt);
            let active_count = self.engine.active_count(speed);
            ParticleRenderer::draw(
                pixmap,
                self.engine.particles(),
                active_count,
                center_x,
                center_y,
                core_color,
                particle_a,
            );
        }

        // Brackets "( )" with outward flying effect from center
        if brk_a > 0.001 {
            SpeedometerBrackets::draw(pixmap, core_color, glow_color, brk_fly, brk_a);
        }

        // Speed digits
        if digit_a > 0.001 {
            SpeedometerDigits::draw(pixmap, fonts, speed, center_x, core_color, digit_a);
        }
    }
}
