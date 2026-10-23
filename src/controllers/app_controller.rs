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

        let wave_tracker = SpeedWaveTracker::new();

        let runtime = Rc::new(RefCell::new(RuntimeState {
            state,
            demo_gps,
            telemetry,
            wave_tracker,
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

                // Update speed-driven undulating dynamic waveform (never stationary)
                let current_speed = runtime.state.current_speed;
                runtime.wave_tracker.update(current_speed, dt);
                let (stroke_path, fill_path, peak_x, peak_y, y1, y2, y4, y5, y6) =
                    runtime.wave_tracker.generate_paths();

                ui.set_chart_stroke_path(stroke_path.into());
                ui.set_chart_fill_path(fill_path.into());
                ui.set_chart_peak_x(peak_x);
                ui.set_chart_peak_y(peak_y);
                ui.set_chart_y1(y1);
                ui.set_chart_y2(y2);
                ui.set_chart_y4(y4);
                ui.set_chart_y5(y5);
                ui.set_chart_y6(y6);

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
    wave_tracker: SpeedWaveTracker,
    config: AppConfig,
    last_time: Instant,
    last_config_check: Instant,
    last_config_mtime: Option<std::time::SystemTime>,
}

pub struct SpeedWaveTracker {
    pub current_smoothed_speed: f32,
    pub wave_phase: f32,
}

impl SpeedWaveTracker {
    pub fn new() -> Self {
        Self {
            current_smoothed_speed: 0.0,
            wave_phase: 0.0,
        }
    }

    pub fn update(&mut self, target_speed: f32, dt: f32) {
        if dt > 0.001 {
            // Smooth speed transitions
            self.current_smoothed_speed =
                self.current_smoothed_speed * 0.90 + target_speed * 0.10;
            // The wave ripples and flows continuously; flow rate accelerates with speed
            let flow_speed = 1.8 + (self.current_smoothed_speed / 15.0);
            self.wave_phase =
                (self.wave_phase + dt * flow_speed).rem_euclid(std::f32::consts::TAU * 100.0);
        }
    }

    /// Generates SVG path commands exactly reproducing the user's reference waveform
    /// which undulates and scales with vehicle speed in real-time (never stationary)
    pub fn generate_paths(&self) -> (String, String, f32, f32, f32, f32, f32, f32, f32) {
        let speed_factor = (self.current_smoothed_speed / 100.0).clamp(0.0, 1.2);
        let t = self.wave_phase;

        let baseline = 210.0;
        // Peaks rise higher as speed increases, and undulate smoothly with wave_phase
        let y3 = (baseline - (48.0 + speed_factor * 115.0) + (t * 2.0).sin() * 7.0)
            .clamp(20.0, 175.0); // Highest active peak with marker bead
        let y1 = (baseline - (28.0 + speed_factor * 65.0) + (t * 2.0 + 1.2).sin() * 5.0)
            .clamp(40.0, 185.0);
        let y2 = (baseline - (12.0 + speed_factor * 30.0) + (t * 2.0 + 2.4).sin() * 4.0)
            .clamp(50.0, 195.0);
        let y4 = (baseline - (16.0 + speed_factor * 38.0) + (t * 2.0 + 3.6).sin() * 4.0)
            .clamp(50.0, 195.0);
        let y5 = (baseline - (36.0 + speed_factor * 85.0) + (t * 2.0 + 4.8).sin() * 6.0)
            .clamp(25.0, 180.0);
        let y0 = (baseline - (18.0 + speed_factor * 40.0) + (t * 2.0).cos() * 4.0)
            .clamp(40.0, 195.0);
        let y6 = (baseline - (22.0 + speed_factor * 50.0) + (t * 2.0 + 1.0).cos() * 4.0)
            .clamp(40.0, 195.0);

        let stroke_path = format!(
            "M 0 {:.1} C 30 {:.1}, 45 {:.1}, 65 {:.1} C 85 {:.1}, 110 {:.1}, 130 {:.1} C 158 {:.1}, 175 {:.1}, 200 {:.1} C 225 {:.1}, 245 {:.1}, 260 {:.1} C 280 {:.1}, 300 {:.1}, 320 {:.1} C 342 {:.1}, 355 {:.1}, 370 {:.1}",
            y0, y0, y1, y1,
            y1, y2, y2,
            y2, y3, y3,
            y3, y4, y4,
            y4, y5, y5,
            y5, y6, y6
        );

        let fill_path = format!("{} L 370 210 L 0 210 Z", stroke_path);
        (stroke_path, fill_path, 200.0, y3, y1, y2, y4, y5, y6)
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

    // Connectivity status (placed on left below speed chart)
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
