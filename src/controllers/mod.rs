// Controllers module facade

pub mod app_controller;
pub mod cli;

pub use app_controller::AppController;
pub use cli::parse_args;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();
    AppController::run(config)
}
