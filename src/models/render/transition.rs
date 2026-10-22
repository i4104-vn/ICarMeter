//! Render transition states and visual parameters models

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
    pub digits_speed: Option<f32>,
    pub digits_alpha: f32,
    pub particles_alpha: f32,
    pub welcome_time: f32,
    pub welcome_alpha: f32,
}
