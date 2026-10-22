// Particle model for inward suction speed visualization

#[derive(Debug, Clone)]
pub struct Particle {
    pub angle: f32, // Fixed radial trajectory angle (straight into center)
    pub dist: f32,  // Current distance from center
    pub size: f32,
    pub speed_factor: f32,
}

impl Particle {
    pub fn new(angle: f32, dist: f32, size: f32, speed_factor: f32) -> Self {
        Self {
            angle,
            dist,
            size,
            speed_factor,
        }
    }
}
