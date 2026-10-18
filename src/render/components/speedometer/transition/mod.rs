//! Center-panel transition state machine.

mod visual_params;

#[allow(unused_imports)]
pub use visual_params::CenterVisualParams;

// Shared timeline for the center welcome animation.
pub const WELCOME_SLEEP_END: f32 = 4.0;
pub const WELCOME_WAKE_END: f32 = 8.0;
pub const WELCOME_END: f32 = WELCOME_WAKE_END;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CenterModeState {
    Welcome(f32),
    SpeedActive,
    FadingSpeedToEmo(f32),
    EmoActive,
    FadingEmoOut(f32),
    FlyingBracketsOut(f32),
    FadingSpeedIn(f32),
}

pub struct CenterTransitionManager {
    pub state: CenterModeState,
    pub idle_zero_timer: f32,
    pub elapsed_time: f32,
}

impl CenterTransitionManager {
    pub fn new() -> Self {
        Self {
            state: CenterModeState::Welcome(0.0),
            idle_zero_timer: 0.0,
            elapsed_time: 0.0,
        }
    }

    pub fn update(&mut self, speed: f32, dt: f32) {
        // Do not jump through the welcome sequence after a slow framebuffer
        // write or a temporary scheduler pause.
        let dt = dt.max(0.0).min(0.05);
        self.elapsed_time += dt;
        if speed <= 0.05 {
            self.idle_zero_timer += dt;
        } else {
            self.idle_zero_timer = 0.0;
        }

        match self.state {
            CenterModeState::Welcome(time) => {
                let next_time = time + dt;
                self.state = if speed > 0.5 {
                    CenterModeState::FlyingBracketsOut(0.0)
                } else if next_time >= WELCOME_END {
                    CenterModeState::EmoActive
                } else {
                    CenterModeState::Welcome(next_time)
                };
            }
            CenterModeState::SpeedActive => {
                if self.idle_zero_timer >= 3.0 {
                    self.state = CenterModeState::FadingSpeedToEmo(0.0);
                }
            }
            CenterModeState::FadingSpeedToEmo(progress) => {
                self.state = if speed > 0.05 {
                    CenterModeState::SpeedActive
                } else {
                    next_state(
                        progress,
                        dt,
                        0.4,
                        CenterModeState::EmoActive,
                        Self::fading_speed_to_emo,
                    )
                };
            }
            CenterModeState::EmoActive => {
                if speed > 0.05 {
                    self.state = CenterModeState::FadingEmoOut(0.0);
                }
            }
            CenterModeState::FadingEmoOut(progress) => {
                self.state = next_state(
                    progress,
                    dt,
                    0.3,
                    CenterModeState::FlyingBracketsOut(0.0),
                    Self::fading_emo_out,
                );
            }
            CenterModeState::FlyingBracketsOut(progress) => {
                self.state = next_state(
                    progress,
                    dt,
                    0.38,
                    CenterModeState::FadingSpeedIn(0.0),
                    Self::flying_brackets_out,
                );
            }
            CenterModeState::FadingSpeedIn(progress) => {
                self.state = next_state(
                    progress,
                    dt,
                    0.25,
                    CenterModeState::SpeedActive,
                    Self::fading_speed_in,
                );
            }
        }
    }

    fn fading_speed_to_emo(progress: f32) -> CenterModeState {
        CenterModeState::FadingSpeedToEmo(progress)
    }

    fn fading_emo_out(progress: f32) -> CenterModeState {
        CenterModeState::FadingEmoOut(progress)
    }

    fn flying_brackets_out(progress: f32) -> CenterModeState {
        CenterModeState::FlyingBracketsOut(progress)
    }

    fn fading_speed_in(progress: f32) -> CenterModeState {
        CenterModeState::FadingSpeedIn(progress)
    }
}

fn next_state(
    progress: f32,
    dt: f32,
    duration: f32,
    completed: CenterModeState,
    continuing: fn(f32) -> CenterModeState,
) -> CenterModeState {
    let next_progress = progress + dt / duration;
    if next_progress >= 1.0 {
        completed
    } else {
        continuing(next_progress)
    }
}
