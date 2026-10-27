//! Render sprite path and frame models

use tiny_skia::{Color, Path};

pub struct ExpressionSprite {
    pub frames: Vec<Vec<ColoredPath>>,
    pub bounds: Option<(f32, f32, f32, f32)>,
    pub frame_width: f32,
}

#[derive(Clone)]
pub struct ColoredPath {
    pub path: Path,
    pub color: Color,
    pub opacity: f32,
}
