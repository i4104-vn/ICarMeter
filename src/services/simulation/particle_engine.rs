// Particle engine: straight-line inward suction physics & density scaling

use crate::models::Particle;

pub const MAX_PARTICLES: usize = 180;

pub struct ParticleEngine {
    particles: Vec<Particle>,
}

impl ParticleEngine {
    pub fn new() -> Self {
        let mut particles = Vec::with_capacity(MAX_PARTICLES);
        for i in 0..MAX_PARTICLES {
            let seed = (i as f32 * 19.17).sin().abs();
            let angle = (i as f32 / MAX_PARTICLES as f32) * std::f32::consts::TAU + (seed * 0.5);
            let dist = 38.0 + seed * 185.0; // Suction field radius up to 223px
            let size = 1.4 + seed * 2.2;
            let speed_factor = 0.75 + seed * 0.55;
            particles.push(Particle::new(angle, dist, size, speed_factor));
        }
        Self { particles }
    }

    /// Number of active particles, scaled proportionally with current speed
    pub fn active_count(&self, speed: f32) -> usize {
        ((speed / 120.0) * 150.0 + 30.0).clamp(30.0, MAX_PARTICLES as f32) as usize
    }

    /// Update particles: inward straight-line suction (no circular rotation)
    pub fn update(&mut self, speed: f32, dt: f32) {
        let inward_speed = 15.0 + (speed * 1.85);
        let active = self.active_count(speed);

        for (idx, p) in self.particles[..active].iter_mut().enumerate() {
            p.dist -= inward_speed * p.speed_factor * dt;

            // When particle enters the center core, respawn at outer boundary
            if p.dist < 32.0 {
                p.dist = 165.0 + ((idx as f32 * 23.7).sin().abs()) * 58.0;
                p.angle = ((idx as f32 * 11.3).cos() * 0.5 + 0.5) * std::f32::consts::TAU;
            }
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }
}
