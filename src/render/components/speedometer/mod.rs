// Speedometer & Emo Companion Center Panel Facade

pub mod brackets;
pub mod digits;
pub mod particles;
pub mod tier;
pub mod transition;
pub mod welcome;

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
use welcome::SpeedometerWelcome;

pub struct SpeedometerPanel {
    engine: ParticleEngine,
    emo: EmoRenderer,
    transitions: CenterTransitionManager,
    visual_speed: f32,
}

impl SpeedometerPanel {
    pub fn new() -> Self {
        Self {
            engine: ParticleEngine::new(),
            emo: EmoRenderer::new(),
            transitions: CenterTransitionManager::new(),
            visual_speed: 0.0,
        }
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

        // If forced expression provided, lock to Emo mode
        if forced_emo.is_some() && self.transitions.state != CenterModeState::EmoActive {
            self.transitions.state = CenterModeState::EmoActive;
        }

        // 1. Update center state machine
        self.transitions.update(speed, dt);
        let params = self.transitions.visual_parameters();
        let render_speed = self.smooth_speed(speed, dt);

        // 2. Draw Startup Welcome sequence if active
        if params.welcome_alpha > 0.001 {
            SpeedometerWelcome::draw(
                pixmap,
                fonts,
                center_x,
                center_y,
                params.welcome_time,
                params.welcome_alpha,
            );
        }

        // 3. Draw Emo digital companion eyes if active/fading
        if params.emo_alpha > 0.001 {
            self.emo.draw(
                pixmap,
                fonts,
                center_x,
                center_y,
                dt,
                self.transitions.elapsed_time,
                self.transitions.idle_zero_timer,
                params.emo_alpha,
                forced_emo,
            );
        }

        // 4. Draw Speedometer elements if active/fading
        let display_speed = params.brackets_speed.unwrap_or(render_speed);
        let digits_speed = params.digits_speed.unwrap_or(render_speed);
        let (core_color, _glow_color) = get_tier_colors(digits_speed);

        // Suction particles
        if params.particles_alpha > 0.001 {
            self.engine.update(digits_speed, dt);
            let active_count = self.engine.active_count(digits_speed);
            ParticleRenderer::draw(
                pixmap,
                self.engine.particles(),
                active_count,
                center_x,
                center_y,
                core_color,
                params.particles_alpha,
            );
        }

        // Brackets "( )" with progressive bottom-to-top fill and tiered color transitions
        if params.brackets_alpha > 0.001 {
            SpeedometerBrackets::draw(
                pixmap,
                display_speed,
                params.brackets_fly,
                params.brackets_alpha,
            );
        }

        // Speed digits
        if params.digits_alpha > 0.001 {
            SpeedometerDigits::draw(
                pixmap,
                fonts,
                digits_speed,
                center_x,
                core_color,
                params.digits_alpha,
            );
        }
    }

    fn smooth_speed(&mut self, target: f32, dt: f32) -> f32 {
        let dt = dt.clamp(0.0, 0.05);
        let response = 1.0 - (-dt * 14.0).exp();
        self.visual_speed += (target.max(0.0) - self.visual_speed) * response;
        if target <= 0.01 && self.visual_speed < 0.05 {
            self.visual_speed = 0.0;
        }
        self.visual_speed
    }
}
