// Services module facade

pub mod display;
pub mod input;
pub mod simulation;

pub use display::create_backend;
pub use simulation::{ParticleEngine, VehicleSimulator};
