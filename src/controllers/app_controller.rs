// Main Application Controller managing state and render cycle on Raspberry Pi OS

use crate::configs::app::AppConfig;
use crate::controllers::demo::DemoGps;
use crate::models::bluetooth::state::BluetoothSharedState;
use crate::models::vehicle::state::VehicleState;
use crate::render::DashboardRenderer;
use crate::services::{FramebufferBackend, TelemetryService, is_app_running};
use std::thread;
use std::time::{Duration, Instant};

pub struct AppController;

impl AppController {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        // 1. Load hardware configuration from single config path
        let config = AppConfig::load();
        eprintln!(
            "[Config] Active configuration path: {}",
            AppConfig::path().display()
        );

        let mut state = VehicleState::default();
        state.tpms_enabled = config.tpms.enabled;
        state.tires.low_pressure_alert = config.tpms.low_pressure_alert;
        state.tires.high_temp_alert = config.tpms.high_temp_alert;
        let mut demo_gps = DemoGps::new();

        // 2. Initialize native Bluetooth RFCOMM server for mobile companion app
        let bt_shared_state = BluetoothSharedState::new();
        let _bt_server = crate::services::BluetoothServer::start(bt_shared_state.clone());

        // 3. Initialize real telemetry service (GPS & Bluetooth sensors)
        let mut telemetry = TelemetryService::new(&config, Some(bt_shared_state.clone()));

        // 4. Initialize UI renderer
        let mut renderer = DashboardRenderer::new()
            .map_err(|e| format!("Renderer initialization failed: {}", e))?;

        // 5. Initialize Raspberry Pi framebuffer
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
                    state.tpms_enabled = new_config.tpms.enabled;
                    state.tires.low_pressure_alert = new_config.tpms.low_pressure_alert;
                    state.tires.high_temp_alert = new_config.tpms.high_temp_alert;
                    telemetry.reload_config(&new_config);
                }
            }

            if let Some(key) = backend.poll_key() {
                if key.eq_ignore_ascii_case(&b'a') {
                    demo_gps.toggle(&mut state);
                } else if key.eq_ignore_ascii_case(&b'm') {
                    state.tpms_enabled = !state.tpms_enabled;
                    eprintln!(
                        "[Display] Mode switched manually: TPMS enabled = {}",
                        state.tpms_enabled
                    );
                } else if key.eq_ignore_ascii_case(&b'b') {
                    state.show_bt_log = !state.show_bt_log;
                    eprintln!(
                        "[Display] Bluetooth log view toggled: show_bt_log = {}",
                        state.show_bt_log
                    );
                }
            }
            if !backend.is_open() {
                break;
            }

            let now = Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32().min(0.1);
            last_time = now;

            // Non-blocking telemetry sync from dedicated background worker thread
            telemetry.update(&mut state);
            demo_gps.update(&mut state, dt);
            state.update_trip(dt);

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
