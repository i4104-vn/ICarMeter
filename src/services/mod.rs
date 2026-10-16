// Services module facade

pub mod display;
pub mod input;
pub mod simulation;

pub use display::create_backend;
pub use input::is_app_running;
#[allow(unused_imports)]
pub use input::{TtyManager, request_app_exit};
pub use simulation::{ParticleEngine, VehicleSimulator};
