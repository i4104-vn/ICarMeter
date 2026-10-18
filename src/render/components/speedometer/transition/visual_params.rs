//! Visual output mapping for the center-panel transition state machine.

use super::{CenterModeState, CenterTransitionManager, WELCOME_SLEEP_END};

#[derive(Debug, Clone, Copy)]
pub struct CenterVisualParams {
    pub emo_alpha: f32,
    pub brackets_fly: f32,
    pub brackets_alpha: f32,
    pub brackets_speed: Option<f32>,
    pub digits_speed: Option<f32>,
    pub digits_alpha: f32,
    pub particles_alpha: f32,
    pub welcome_time: f32,
    pub welcome_alpha: f32,
}

impl CenterTransitionManager {
    pub fn visual_parameters(&self) -> CenterVisualParams {
        match self.state {
            CenterModeState::Welcome(time) => welcome_parameters(time),
            CenterModeState::SpeedActive => CenterVisualParams {
                emo_alpha: 0.0,
                brackets_fly: 1.0,
                brackets_alpha: 1.0,
                brackets_speed: None,
                digits_speed: None,
                digits_alpha: 1.0,
                particles_alpha: 1.0,
                welcome_time: 0.0,
                welcome_alpha: 0.0,
            },
            CenterModeState::FadingSpeedToEmo(progress) => {
                let emo_alpha = progress.clamp(0.0, 1.0);
                let speed_alpha = (1.0 - progress).clamp(0.0, 1.0);
                CenterVisualParams {
                    emo_alpha,
                    brackets_fly: 1.0,
                    brackets_alpha: speed_alpha,
                    brackets_speed: None,
                    digits_speed: None,
                    digits_alpha: speed_alpha,
                    particles_alpha: speed_alpha,
                    welcome_time: 0.0,
                    welcome_alpha: 0.0,
                }
            }
            CenterModeState::EmoActive => CenterVisualParams {
                emo_alpha: 1.0,
                brackets_fly: 0.0,
                brackets_alpha: 0.0,
                brackets_speed: None,
                digits_speed: None,
                digits_alpha: 0.0,
                particles_alpha: 0.0,
                welcome_time: 0.0,
                welcome_alpha: 0.0,
            },
            CenterModeState::FadingEmoOut(progress) => CenterVisualParams {
                emo_alpha: (1.0 - progress).clamp(0.0, 1.0),
                brackets_fly: 0.0,
                brackets_alpha: 0.0,
                brackets_speed: None,
                digits_speed: None,
                digits_alpha: 0.0,
                particles_alpha: 0.0,
                welcome_time: 0.0,
                welcome_alpha: 0.0,
            },
            CenterModeState::FlyingBracketsOut(progress) => {
                let fly = progress.clamp(0.0, 1.0);
                CenterVisualParams {
                    emo_alpha: 0.0,
                    brackets_fly: fly,
                    brackets_alpha: (fly * 1.5).min(1.0),
                    brackets_speed: None,
                    digits_speed: None,
                    digits_alpha: 0.0,
                    particles_alpha: 0.0,
                    welcome_time: 0.0,
                    welcome_alpha: 0.0,
                }
            }
            CenterModeState::FadingSpeedIn(progress) => {
                let speed_alpha = progress.clamp(0.0, 1.0);
                CenterVisualParams {
                    emo_alpha: 0.0,
                    brackets_fly: 1.0,
                    brackets_alpha: 1.0,
                    brackets_speed: None,
                    digits_speed: None,
                    digits_alpha: speed_alpha,
                    particles_alpha: speed_alpha,
                    welcome_time: 0.0,
                    welcome_alpha: 0.0,
                }
            }
        }
    }
}

fn welcome_parameters(time: f32) -> CenterVisualParams {
    CenterVisualParams {
        emo_alpha: 0.0,
        brackets_fly: 0.0,
        brackets_alpha: 0.0,
        brackets_speed: None,
        digits_speed: None,
        digits_alpha: 0.0,
        particles_alpha: 0.0,
        welcome_time: time,
        // Fade the center in during the first part of the sleep animation.
        welcome_alpha: smoothstep((time / WELCOME_SLEEP_END).clamp(0.0, 1.0)),
    }
}

fn smoothstep(progress: f32) -> f32 {
    let t = progress.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
