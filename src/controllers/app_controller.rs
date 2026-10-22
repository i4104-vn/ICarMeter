// Main Application Controller managing state and render cycle on Raspberry Pi OS

use crate::configs::AppConfig;
use crate::controllers::demo::DemoGps;
use crate::models::VehicleState;
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

        let mut state = VehicleState::new();
        state.tpms_enabled = config.tpms.enabled;
        state.tires.low_pressure_alert = config.tpms.low_pressure_alert;
        state.tires.high_temp_alert = config.tpms.high_temp_alert;
        let mut demo_gps = DemoGps::new();

        // 2. Initialize real telemetry service (GPS & Bluetooth sensors)
        let mut telemetry = TelemetryService::new(&config);

        // 3. Initialize native Bluetooth RFCOMM server for mobile companion app
        let bt_shared_state = crate::services::BluetoothSharedState::new();
        let _bt_server = crate::services::BluetoothServer::start(bt_shared_state.clone());

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
        let mut last_tpms_shm_write = Instant::now();

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

            // Update real vehicle telemetry from GPS and Bluetooth sensors
            telemetry.update(&mut state);
            demo_gps.update(&mut state, dt);
            state.update_trip(dt);

            // Expose real-time TPMS telemetry to native Bluetooth server and shared memory
            if last_tpms_shm_write.elapsed() >= Duration::from_millis(250) {
                last_tpms_shm_write = Instant::now();
                bt_shared_state.set_tires(&state.tires);
                let tpms_json = format!(
                    "{{\"fl\":{:.2},\"fr\":{:.2},\"rl\":{:.2},\"rr\":{:.2},\"fl_temp\":{:.1},\"fr_temp\":{:.1},\"rl_temp\":{:.1},\"rr_temp\":{:.1},\"alert\":{:.2},\"high_temp_alert\":{:.1}}}",
                    state.tires.fl,
                    state.tires.fr,
                    state.tires.rl,
                    state.tires.rr,
                    state.tires.fl_temp,
                    state.tires.fr_temp,
                    state.tires.rl_temp,
                    state.tires.rr_temp,
                    state.tires.low_pressure_alert,
                    state.tires.high_temp_alert
                );
                let _ = std::fs::write("/dev/shm/icarmeter_tpms_telemetry", tpms_json);
            }

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
