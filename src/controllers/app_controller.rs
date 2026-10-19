// Main Application Controller managing state and render cycle on Raspberry Pi OS

use crate::configs::AppConfig;
use crate::controllers::demo::DemoSpeed;
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
        state.tires.low_pressure_alert = config.tpms.low_pressure_alert;
        state.tires.high_temp_alert = config.tpms.high_temp_alert;
        let mut demo_speed = DemoSpeed::new();

        // 2. Initialize real telemetry service (GPS & Bluetooth sensors)
        let mut telemetry = TelemetryService::new(&config);

        // 3. Initialize UI renderer
        let mut renderer = DashboardRenderer::new()
            .map_err(|e| format!("Renderer initialization failed: {}", e))?;

        // 4. Initialize Raspberry Pi framebuffer
        let mut backend = FramebufferBackend::open(&config.display.framebuffer_device)
            .map_err(|e| format!("Failed to initialize framebuffer backend: {}", e))?;

        let target_fps = config.display.target_fps.clamp(15, 240);
        let frame_duration = Duration::from_micros(1_000_000 / target_fps);
        let mut last_time = Instant::now();
        let mut last_config_check = Instant::now();
        let mut last_config_mtime = AppConfig::last_modified();

        while backend.is_open() && is_app_running() {
            // Check and re-read config.toml dynamically in real time
            if last_config_check.elapsed() >= Duration::from_millis(500) {
                last_config_check = Instant::now();
                let current_mtime = AppConfig::last_modified();
                if current_mtime != last_config_mtime {
                    last_config_mtime = current_mtime;
                    let new_config = AppConfig::load();
                    eprintln!(
                        "[Config] Real-time config.toml update detected:\n  TPMS FL: {} | FR: {}\n  TPMS RL: {} | RR: {}\n  Alert: {:.1} bar | {:.1} °C | Enabled: {}",
                        new_config.tpms.fl_sensor,
                        new_config.tpms.fr_sensor,
                        new_config.tpms.rl_sensor,
                        new_config.tpms.rr_sensor,
                        new_config.tpms.low_pressure_alert,
                        new_config.tpms.high_temp_alert,
                        new_config.tpms.enabled
                    );
                    state.tires.low_pressure_alert = new_config.tpms.low_pressure_alert;
                    state.tires.high_temp_alert = new_config.tpms.high_temp_alert;
                    telemetry.reload_config(&new_config);
                }
            }

            if let Some(key) = backend.poll_key()
                && key.eq_ignore_ascii_case(&b'a')
            {
                demo_speed.toggle(&mut state);
            }
            if !backend.is_open() {
                break;
            }

            let now = Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32().min(0.1);
            last_time = now;

            // Update real vehicle telemetry from GPS and Bluetooth sensors
            telemetry.update(&mut state, dt);
            demo_speed.update(&mut state, dt);

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
