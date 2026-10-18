// Controllers module facade

pub mod app_controller;
pub mod demo;

pub use app_controller::AppController;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    AppController::run()
}
