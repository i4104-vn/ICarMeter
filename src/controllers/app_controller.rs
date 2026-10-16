// Main Application Controller managing state and render cycle

use super::cli::CliConfig;
use crate::configs::display::{PHYSICAL_HEIGHT_MM, PHYSICAL_WIDTH_MM, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::models::VehicleState;
use crate::render::DashboardRenderer;
use crate::services::{VehicleSimulator, create_backend, is_app_running};
use std::thread;
use std::time::{Duration, Instant};

pub struct AppController;

impl AppController {
    pub fn run(config: CliConfig) -> Result<(), Box<dyn std::error::Error>> {
        if config.should_exit {
            return Ok(());
        }

        let mut state = VehicleState::new();
        if let Some(speed) = config.initial_speed {
            state.current_speed = speed;
        }

        // Initialize UI renderer
        let mut renderer = DashboardRenderer::new()
            .map_err(|e| format!("Renderer initialization failed: {}", e))?;

        if config.initial_speed == Some(0.0) || config.forced_emo.is_some() {
            renderer.set_immediate_emo();
        }

        // If snapshot mode requested, render once and save
        if let Some(path) = config.snapshot_path {
            println!("[Dashboard] Rendering frame snapshot to '{}'...", path);
            renderer.render(&state, 0.016, config.forced_emo);
            renderer.save_png(&path)?;
            println!("[Dashboard] Snapshot saved successfully!");
            return Ok(());
        }

        // Initialize display backend
        let mut backend = create_backend(config.backend_type)
            .map_err(|e| format!("Failed to initialize display backend: {}", e))?;

        println!("============================================================");
        println!("  Starting Car Digital Dashboard");
        println!(
            "  Screen: 7-inch ({:.0}mm x {:.0}mm, {}x{} pixels)",
            PHYSICAL_WIDTH_MM, PHYSICAL_HEIGHT_MM, SCREEN_WIDTH, SCREEN_HEIGHT
        );
        println!("  Backend: {}", backend.backend_name());
        println!("  Target FPS: {}", config.target_fps);
        println!("  Mode: Fully Automatic Telemetry & Emo Companion");
        println!("============================================================");

        let frame_duration = Duration::from_micros(1_000_000 / config.target_fps);
        let mut last_time = Instant::now();

        while backend.is_open() && is_app_running() {
            let now = Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32().min(0.1);
            last_time = now;

            // 1. Update vehicle telemetry / automatic simulation (unless locked by --speed)
            if config.initial_speed.is_none() && config.forced_emo.is_none() {
                VehicleSimulator::update(&mut state, dt);
            }

            // 2. Render frame to pixel buffer
            let buffer = renderer.render(&state, dt, config.forced_emo);

            // 3. Present to display (minifb window, DRM/KMS dumb buffer, or /dev/fb0)
            if let Err(e) = backend.present(buffer) {
                eprintln!("[Dashboard] Present error: {}", e);
                break;
            }

            // 4. Frame timing limiter
            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }

        println!("[Dashboard] Terminated cleanly.");
        Ok(())
    }
}
