// Main application controller for Slint and the telemetry services.

use crate::configs::app::AppConfig;
use crate::controllers::demo::DemoGps;
use crate::models::bluetooth::BluetoothSharedState;
use crate::models::vehicle::VehicleState;
use crate::services::{TelemetryService, init_signal_handlers, is_app_running};
use slint::{ComponentHandle, Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

slint::include_modules!();

pub struct AppController;

impl AppController {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        init_signal_handlers();
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
        let demo_gps = DemoGps::new();

        // 2. Initialize native Bluetooth RFCOMM server for mobile companion app
        let bt_shared_state = BluetoothSharedState::new();
        let _bt_server = crate::services::BluetoothServer::start(bt_shared_state.clone());

        // 3. Initialize real telemetry service (GPS & Bluetooth sensors)
        let telemetry = TelemetryService::new(&config, Some(bt_shared_state.clone()));

        let ui = AppWindow::new()?;
        let emo_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| (duration.subsec_nanos() % 65_521) as i32)
            .unwrap_or(1)
            .max(1);
        ui.set_emo_random_seed(emo_seed);

        let accel_tracker = AccelDynamicsTracker::new();

        let runtime = Rc::new(RefCell::new(RuntimeState {
            state,
            demo_gps,
            telemetry,
            accel_tracker,
            config,
            last_time: Instant::now(),
            last_config_check: Instant::now(),
            last_config_mtime: AppConfig::last_modified(),
        }));

        ui.on_toggle_mode({
            let runtime = runtime.clone();
            move || {
                let mut runtime = runtime.borrow_mut();
                runtime.state.tpms_enabled = !runtime.state.tpms_enabled;
            }
        });
        ui.on_toggle_bt_log({
            let runtime = runtime.clone();
            move || {
                let mut runtime = runtime.borrow_mut();
                runtime.state.show_bt_log = !runtime.state.show_bt_log;
            }
        });
        ui.on_demo_gps({
            let runtime = runtime.clone();
            move || {
                let mut runtime = runtime.borrow_mut();
                let RuntimeState {
                    state, demo_gps, ..
                } = &mut *runtime;
                demo_gps.toggle(state);
            }
        });
        ui.on_quit({
            let ui = ui.as_weak();
            move || {
                if let Some(ui) = ui.upgrade() {
                    ui.hide().ok();
                }
            }
        });

        let timer = Timer::default();
        let target_fps = runtime.borrow().config.display.target_fps.clamp(15, 240);
        let interval = Duration::from_micros(1_000_000 / target_fps);
        timer.start(TimerMode::Repeated, interval, {
            let ui = ui.as_weak();
            let runtime = runtime.clone();
            move || {
                let Some(ui) = ui.upgrade() else { return };
                if !is_app_running() {
                    ui.hide().ok();
                    return;
                }
                let mut runtime = runtime.borrow_mut();
                update_config(&mut runtime);
                let dt = runtime.last_time.elapsed().as_secs_f32().min(0.1);
                runtime.last_time = Instant::now();
                {
                    let RuntimeState {
                        state, telemetry, ..
                    } = &mut *runtime;
                    telemetry.update(state);
                }
                {
                    let RuntimeState {
                        state, demo_gps, ..
                    } = &mut *runtime;
                    demo_gps.update(state, dt);
                }
                runtime.state.update_trip(dt);

                // Update acceleration/deceleration telemetry and dynamic waveform
                let current_speed = runtime.state.current_speed;
                runtime.accel_tracker.update(current_speed, dt);
                let (stroke_path, fill_path, peak_x, peak_y) = runtime.accel_tracker.generate_paths();
                ui.set_chart_stroke_path(stroke_path.into());
                ui.set_chart_fill_path(fill_path.into());
                ui.set_chart_peak_x(peak_x);
                ui.set_chart_peak_y(peak_y);
                ui.set_current_accel(runtime.accel_tracker.current_accel);

                sync_ui(&ui, &runtime.state);
            }
        });

        sync_ui(&ui, &runtime.borrow().state);
        ui.show()?;
        ui.run()?;
        timer.stop();

        println!("[Dashboard] Terminated cleanly.");
        Ok(())
    }
}

struct RuntimeState {
    state: VehicleState,
    demo_gps: DemoGps,
    telemetry: TelemetryService,
    accel_tracker: AccelDynamicsTracker,
    config: AppConfig,
    last_time: Instant,
    last_config_check: Instant,
    last_config_mtime: Option<std::time::SystemTime>,
}

pub struct AccelDynamicsTracker {
    pub last_speed: f32,
    pub current_accel: f32,
    pub wave_phase: f32,
}

impl AccelDynamicsTracker {
    pub fn new() -> Self {
        Self {
            last_speed: 0.0,
            current_accel: 0.0,
            wave_phase: 0.0,
        }
    }

    pub fn update(&mut self, current_speed: f32, dt: f32) {
        if dt > 0.001 {
            let speed_delta_ms = (current_speed - self.last_speed) / 3.6;
            let raw_accel = speed_delta_ms / dt;
            // Smooth EMA filter to produce organic, readable vehicle acceleration values
            self.current_accel = self.current_accel * 0.88 + raw_accel.clamp(-7.0, 7.0) * 0.12;
            self.wave_phase = (self.wave_phase + dt * 1.5).rem_euclid(std::f32::consts::TAU);
        }
        self.last_speed = current_speed;
    }

    /// Generates SVG path commands exactly reproducing the user's reference waveform
    /// with glowing neon red curves and dynamic height response to acceleration/braking
    pub fn generate_paths(&self) -> (String, String, f32, f32) {
        let a = self.current_accel;
        let p = (self.wave_phase).sin() * 1.5;

        // Base wave node coordinates modeled directly after reference image
        let y0 = 160.0 - a * 1.5 + p * 0.5;
        let y1 = (120.0 - a * 5.5 + p).clamp(60.0, 180.0);
        let y2 = 165.0 - a * 2.5 + p * 0.5;
        let y3 = (50.0 - a * 11.0 - p * 1.2).clamp(20.0, 100.0); // Highest active peak with marker bead
        let y4 = 135.0 - a * 3.5 + p * 0.8;
        let y5 = (30.0 - a * 7.5 - p).clamp(15.0, 90.0);         // Secondary peak on the right
        let y6 = 100.0 - a * 2.0 + p * 0.5;

        let stroke_path = format!(
            "M 0 {:.1} C 30 {:.1}, 45 {:.1}, 65 {:.1} C 85 {:.1}, 110 {:.1}, 130 {:.1} C 158 {:.1}, 175 {:.1}, 200 {:.1} C 225 {:.1}, 245 {:.1}, 260 {:.1} C 280 {:.1}, 300 {:.1}, 320 {:.1} C 342 {:.1}, 355 {:.1}, 370 {:.1}",
            y0, y0, y1, y1,
            y1, y2, y2,
            y2, y3, y3,
            y3, y4, y4,
            y4, y5, y5,
            y5, y6, y6
        );

        let fill_path = format!("{} L 370 220 L 0 220 Z", stroke_path);
        (stroke_path, fill_path, 200.0, y3)
    }
}

fn update_config(runtime: &mut RuntimeState) {
    if runtime.last_config_check.elapsed() < Duration::from_millis(500) {
        return;
    }
    runtime.last_config_check = Instant::now();
    let current_mtime = AppConfig::last_modified();
    if current_mtime == runtime.last_config_mtime {
        return;
    }
    runtime.last_config_mtime = current_mtime;
    let new_config = AppConfig::load();
    runtime.state.tpms_enabled = new_config.tpms.enabled;
    runtime.state.tires.low_pressure_alert = new_config.tpms.low_pressure_alert;
    runtime.state.tires.high_temp_alert = new_config.tpms.high_temp_alert;
    runtime.telemetry.reload_config(&new_config);
    runtime.config = new_config;
}

fn sync_ui(ui: &AppWindow, state: &VehicleState) {
    ui.set_speed(state.current_speed);
    ui.set_driving_time(state.formatted_driving_time().into());
    ui.set_distance(state.formatted_distance().into());
    ui.set_max_speed(state.formatted_max_speed().into());
    ui.set_avg_speed(state.formatted_avg_speed().into());
    ui.set_tpms_enabled(state.tpms_enabled);
    ui.set_fl_pressure(state.tires.fl);
    ui.set_fr_pressure(state.tires.fr);
    ui.set_rl_pressure(state.tires.rl);
    ui.set_rr_pressure(state.tires.rr);
    ui.set_fl_temp(state.tires.fl_temp);
    ui.set_fr_temp(state.tires.fr_temp);
    ui.set_rl_temp(state.tires.rl_temp);
    ui.set_rr_temp(state.tires.rr_temp);
    ui.set_alert_pressure(state.tires.low_pressure_alert);
    ui.set_alert_temp(state.tires.high_temp_alert);

    // Connectivity status (matching trippanel style)
    ui.set_bt_active(state.bt_active);
    ui.set_bt_device_name(
        state
            .bt_device_name
            .as_deref()
            .unwrap_or("Bluetooth")
            .into(),
    );
    ui.set_gps_connected(state.gps_connected);
    ui.set_show_bt_log(state.show_bt_log);
    if let Some(last_log) = state.bt_logs.last() {
        ui.set_latest_bt_log(last_log.as_str().into());
    }
}
