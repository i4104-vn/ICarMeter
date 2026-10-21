// Entry point for the Car Digital Dashboard

mod configs;
mod controllers;
mod models;
mod render;
mod services;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("icarmeter {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("ICarMeter Digital Dashboard v{}", env!("CARGO_PKG_VERSION"));
        println!("Usage: icarmeter [options]");
        return Ok(());
    }

    controllers::run()
}
