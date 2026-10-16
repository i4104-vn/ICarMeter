// Entry point for the Car Digital Dashboard

mod configs;
mod controllers;
mod models;
mod render;
mod services;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    controllers::run()
}
