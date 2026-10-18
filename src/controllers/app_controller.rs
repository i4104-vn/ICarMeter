// Main Application Controller managing state and render cycle on Raspberry Pi OS

use crate::configs::AppConfig;
use crate::configs::display::{PHYSICAL_HEIGHT_MM, PHYSICAL_WIDTH_MM, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::models::VehicleState;
use crate::render::DashboardRenderer;
use crate::services::{FramebufferBackend, TelemetryService, is_app_running};
use std::thread;
use std::time::{Duration, Instant};

pub struct AppController;

impl AppController {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        // 1. Load hardware configuration from config.toml
        let config = AppConfig::load();

        let mut state = VehicleState::new();

        // 2. Initialize real telemetry service (GPS & Bluetooth sensors)
        let mut telemetry = TelemetryService::new(&config);

        // 3. Initialize UI renderer
        let mut renderer = DashboardRenderer::new()
            .map_err(|e| format!("Renderer initialization failed: {}", e))?;

        // 4. Initialize Raspberry Pi framebuffer
        let mut backend = FramebufferBackend::open(&config.display.framebuffer_device)
            .map_err(|e| format!("Failed to initialize framebuffer backend: {}", e))?;

        eprintln!("============================================================");
        eprintln!("  Car Digital Dashboard - Raspberry Pi OS (/dev/fb0)");
        eprintln!(
            "  Screen: 7-inch ({:.0}mm x {:.0}mm, {}x{} pixels)",
            PHYSICAL_WIDTH_MM, PHYSICAL_HEIGHT_MM, SCREEN_WIDTH, SCREEN_HEIGHT
        );
        eprintln!("  Target FPS: {}", config.display.target_fps);
        eprintln!("  GPS device: {} (enabled: {})", config.gps.device, config.gps.enabled);
        eprintln!(
            "  TPMS 4-Tire Wireless BLE ({}, enabled: {}):",
            config.tpms.adapter, config.tpms.enabled
        );
        eprintln!("    FL: {} | FR: {}", config.tpms.fl_sensor, config.tpms.fr_sensor);
        eprintln!("    RL: {} | RR: {}", config.tpms.rl_sensor, config.tpms.rr_sensor);
        eprintln!("============================================================");

        let target_fps = config.display.target_fps.clamp(15, 240);
        let frame_duration = Duration::from_micros(1_000_000 / target_fps);
        let mut last_time = Instant::now();

        while backend.is_open() && is_app_running() {
            let now = Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32().min(0.1);
            last_time = now;

            // Update real vehicle telemetry from GPS and Bluetooth sensors
            telemetry.update(&mut state, dt);

            // Render frame to pixel buffer
            let buffer = renderer.render(&state, dt);

            // Present to framebuffer
            if let Err(e) = backend.present(buffer) {
                eprintln!("[Dashboard] Present error: {}", e);
                break;
            }

            // Frame timing limiter
            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }

        println!("[Dashboard] Terminated cleanly.");
        Ok(())
    }
}
