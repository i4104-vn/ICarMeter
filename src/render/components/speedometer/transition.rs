// Center panel transition manager: handles welcome animation on startup,
// 3s idle at 0 km/h, Emo eyes fade, outward bracket flight, and speed digits fade in.

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

#[derive(Debug, Clone, Copy)]
pub struct CenterVisualParams {
    pub emo_alpha: f32,
    pub brackets_fly: f32,
    pub brackets_alpha: f32,
    pub brackets_speed: Option<f32>,
    pub digits_alpha: f32,
    pub particles_alpha: f32,
    pub welcome_time: f32,
    pub welcome_alpha: f32,
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
        self.elapsed_time += dt;

        // Continuously accumulate stationary duration at 0 km/h
        if speed <= 0.05 {
            self.idle_zero_timer += dt;
        } else {
            self.idle_zero_timer = 0.0;
        }

        match self.state {
            CenterModeState::Welcome(t) => {
                let next_t = t + dt;
                if speed > 0.5 {
                    // Vehicle moves, smoothly jump to speedometer
                    self.state = CenterModeState::FlyingBracketsOut(0.0);
                } else if next_t >= 4.2 {
                    // Welcome sequence finished, transition to idle Emo companion
                    self.state = CenterModeState::EmoActive;
                } else {
                    self.state = CenterModeState::Welcome(next_t);
                }
            }
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

    /// Computes visual parameters for all center panel elements
    pub fn visual_parameters(&self) -> CenterVisualParams {
        match self.state {
            CenterModeState::Welcome(t) => {
                let welcome_a = 1.0;
                let emo_a = 0.0;

                // Gauge sweep calculation (0.0s to 1.5s)
                let (sweep_speed, sweep_alpha) = if t < 1.5 {
                    let p = if t < 0.75 {
                        (t / 0.75).powf(0.85)
                    } else {
                        ((1.35 - t) / 0.6).clamp(0.0, 1.0)
                    };
                    let a = if t < 0.2 {
                        t / 0.2
                    } else if t > 1.1 {
                        ((1.5 - t) / 0.4).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    (Some(p * 38.0), a)
                } else {
                    (None, 0.0)
                };

                CenterVisualParams {
                    emo_alpha: emo_a,
                    brackets_fly: 1.0,
                    brackets_alpha: sweep_alpha,
                    brackets_speed: sweep_speed,
                    digits_alpha: 0.0,
                    particles_alpha: 0.0,
                    welcome_time: t,
                    welcome_alpha: welcome_a,
                }
            }
            CenterModeState::SpeedActive => CenterVisualParams {
                emo_alpha: 0.0,
                brackets_fly: 1.0,
                brackets_alpha: 1.0,
                brackets_speed: None,
                digits_alpha: 1.0,
                particles_alpha: 1.0,
                welcome_time: 0.0,
                welcome_alpha: 0.0,
            },
            CenterModeState::FadingSpeedToEmo(p) => {
                let emo_a = p.clamp(0.0, 1.0);
                let speed_a = (1.0 - p).clamp(0.0, 1.0);
                CenterVisualParams {
                    emo_alpha: emo_a,
                    brackets_fly: 1.0,
                    brackets_alpha: speed_a,
                    brackets_speed: None,
                    digits_alpha: speed_a,
                    particles_alpha: speed_a,
                    welcome_time: 0.0,
                    welcome_alpha: 0.0,
                }
            }
            CenterModeState::EmoActive => CenterVisualParams {
                emo_alpha: 1.0,
                brackets_fly: 0.0,
                brackets_alpha: 0.0,
                brackets_speed: None,
                digits_alpha: 0.0,
                particles_alpha: 0.0,
                welcome_time: 0.0,
                welcome_alpha: 0.0,
            },
            CenterModeState::FadingEmoOut(p) => {
                let emo_a = (1.0 - p).clamp(0.0, 1.0);
                CenterVisualParams {
                    emo_alpha: emo_a,
                    brackets_fly: 0.0,
                    brackets_alpha: 0.0,
                    brackets_speed: None,
                    digits_alpha: 0.0,
                    particles_alpha: 0.0,
                    welcome_time: 0.0,
                    welcome_alpha: 0.0,
                }
            }
            CenterModeState::FlyingBracketsOut(p) => {
                let fly_p = p.clamp(0.0, 1.0);
                let brk_a = (fly_p * 1.5).min(1.0);
                CenterVisualParams {
                    emo_alpha: 0.0,
                    brackets_fly: fly_p,
                    brackets_alpha: brk_a,
                    brackets_speed: None,
                    digits_alpha: 0.0,
                    particles_alpha: 0.0,
                    welcome_time: 0.0,
                    welcome_alpha: 0.0,
                }
            }
            CenterModeState::FadingSpeedIn(p) => {
                let spd_a = p.clamp(0.0, 1.0);
                CenterVisualParams {
                    emo_alpha: 0.0,
                    brackets_fly: 1.0,
                    brackets_alpha: 1.0,
                    brackets_speed: None,
                    digits_alpha: spd_a,
                    particles_alpha: spd_a,
                    welcome_time: 0.0,
                    welcome_alpha: 0.0,
                }
            }
        }
    }
}
