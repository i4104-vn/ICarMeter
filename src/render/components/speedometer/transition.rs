// Center panel transition manager: handles 3s idle at 0 km/h, Emo eyes fade, outward bracket flight, and speed digits fade in

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CenterModeState {
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
            state: CenterModeState::SpeedActive,
            idle_zero_timer: 0.0,
            elapsed_time: 0.0,
        }
    }

    pub fn update(&mut self, speed: f32, dt: f32) {
        self.elapsed_time += dt;

        // Continuously accumulate stationary duration at 0 km/h
        if speed <= 0.05 {
            self.idle_zero_timer += dt;
        } else {
            self.idle_zero_timer = 0.0;
        }

        match self.state {
            CenterModeState::SpeedActive => {
                if self.idle_zero_timer >= 3.0 {
                    self.state = CenterModeState::FadingSpeedToEmo(0.0);
                }
            }
            CenterModeState::FadingSpeedToEmo(p) => {
                if speed > 0.05 {
                    self.state = CenterModeState::SpeedActive;
                } else {
                    let next_p = p + dt / 0.4;
                    if next_p >= 1.0 {
                        self.state = CenterModeState::EmoActive;
                    } else {
                        self.state = CenterModeState::FadingSpeedToEmo(next_p);
                    }
                }
            }
            CenterModeState::EmoActive => {
                if speed > 0.05 {
                    // When speed starts increasing:
                    // 1. Emo eyes fade out
                    self.state = CenterModeState::FadingEmoOut(0.0);
                }
            }
            CenterModeState::FadingEmoOut(p) => {
                let next_p = p + dt / 0.3;
                if next_p >= 1.0 {
                    // 2. Brackets fly outward from center
                    self.state = CenterModeState::FlyingBracketsOut(0.0);
                } else {
                    self.state = CenterModeState::FadingEmoOut(next_p);
                }
            }
            CenterModeState::FlyingBracketsOut(p) => {
                let next_p = p + dt / 0.38;
                if next_p >= 1.0 {
                    // 3. Speed digits fade in
                    self.state = CenterModeState::FadingSpeedIn(0.0);
                } else {
                    self.state = CenterModeState::FlyingBracketsOut(next_p);
                }
            }
            CenterModeState::FadingSpeedIn(p) => {
                let next_p = p + dt / 0.25;
                if next_p >= 1.0 {
                    self.state = CenterModeState::SpeedActive;
                } else {
                    self.state = CenterModeState::FadingSpeedIn(next_p);
                }
            }
        }
    }

    /// Returns (emo_alpha, brackets_fly_progress, brackets_alpha, speed_digits_alpha, particles_alpha)
    pub fn visual_parameters(&self) -> (f32, f32, f32, f32, f32) {
        match self.state {
            CenterModeState::SpeedActive => (0.0, 1.0, 1.0, 1.0, 1.0),
            CenterModeState::FadingSpeedToEmo(p) => {
                let emo_a = p.clamp(0.0, 1.0);
                let speed_a = (1.0 - p).clamp(0.0, 1.0);
                (emo_a, 1.0, speed_a, speed_a, speed_a)
            }
            CenterModeState::EmoActive => (1.0, 0.0, 0.0, 0.0, 0.0),
            CenterModeState::FadingEmoOut(p) => {
                let emo_a = (1.0 - p).clamp(0.0, 1.0);
                (emo_a, 0.0, 0.0, 0.0, 0.0)
            }
            CenterModeState::FlyingBracketsOut(p) => {
                let fly_p = p.clamp(0.0, 1.0);
                let brk_a = (fly_p * 1.5).min(1.0);
                (0.0, fly_p, brk_a, 0.0, 0.0)
            }
            CenterModeState::FadingSpeedIn(p) => {
                let spd_a = p.clamp(0.0, 1.0);
                (0.0, 1.0, 1.0, spd_a, spd_a)
            }
        }
    }
}
