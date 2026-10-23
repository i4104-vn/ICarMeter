// Main application controller for Slint and the telemetry services.

use crate::configs::app::AppConfig;
use crate::controllers::demo::DemoGps;
use crate::models::bluetooth::BluetoothSharedState;
use crate::models::vehicle::VehicleState;
use crate::services::{init_signal_handlers, is_app_running, TelemetryService};
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

                // Add the latest real speed sample to the rolling chart history.
                let current_speed = runtime.state.current_speed;
                if runtime.wave_tracker.update(current_speed, dt) {
                    let (
                        stroke_path,
                        area_path,
                        endpoint_x,
                        endpoint_y,
                        endpoint_visible,
                        y1,
                        y2,
                        y4,
                        y5,
                        y6,
                    ) = runtime.wave_tracker.generate_paths();

                    ui.set_chart_stroke_path(stroke_path.into());
                    ui.set_chart_area_path(area_path.into());
                    ui.set_chart_endpoint_x(endpoint_x);
                    ui.set_chart_endpoint_y(endpoint_y);
                    ui.set_chart_endpoint_visible(endpoint_visible);
                    ui.set_chart_y1(y1);
                    ui.set_chart_y2(y2);
                    ui.set_chart_y4(y4);
                    ui.set_chart_y5(y5);
                    ui.set_chart_y6(y6);
                }

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
    sample_elapsed: f32,
    samples: Vec<f32>,
}

impl SpeedWaveTracker {
    pub fn new() -> Self {
        Self {
            current_smoothed_speed: 0.0,
            sample_elapsed: 0.0,
            samples: vec![0.0; 48],
        }
    }

    pub fn update(&mut self, target_speed: f32, dt: f32) -> bool {
        let mut chart_changed = false;
        if dt > 0.001 {
            // Smooth only the incoming value so the line remains readable while
            // still representing the vehicle speed rather than an independent wave.
            let target_speed = target_speed.max(0.0);
            let smoothing = (dt * 6.0).clamp(0.0, 1.0);
            self.current_smoothed_speed += (target_speed - self.current_smoothed_speed) * smoothing;

            self.sample_elapsed += dt;
            while self.sample_elapsed >= 0.12 {
                self.sample_elapsed -= 0.12;
                self.samples.rotate_left(1);
                let last = self.samples.len() - 1;
                self.samples[last] = self.current_smoothed_speed;
                chart_changed = true;
            }
        }
        chart_changed
    }

    /// Converts the rolling speed samples into a smooth chart and a very
    /// transparent shadow area below it.
    /// A zero-speed history therefore stays exactly on the bottom baseline.
    pub fn generate_paths(&self) -> (String, String, f32, f32, bool, f32, f32, f32, f32, f32) {
        const CHART_WIDTH: f32 = 370.0;
        const CHART_BASELINE: f32 = 205.0;
        const CHART_TOP: f32 = 26.0;
        const CHART_SCALE_SPEED: f32 = 60.0;

        let y_for_speed = |speed: f32| {
            CHART_BASELINE
                - (speed.max(0.0) / CHART_SCALE_SPEED).clamp(0.0, 1.0)
                    * (CHART_BASELINE - CHART_TOP)
        };
        let points: Vec<(f32, f32)> = self
            .samples
            .iter()
            .enumerate()
            .map(|(index, speed)| {
                let x = index as f32 * CHART_WIDTH / (self.samples.len() - 1) as f32;
                (x, y_for_speed(*speed))
            })
            .collect();

        let mut stroke_path = format!("M {:.1} {:.1}", points[0].0, points[0].1);
        for index in 0..points.len() - 1 {
            let p0 = if index == 0 {
                points[0]
            } else {
                points[index - 1]
            };
            let p1 = points[index];
            let p2 = points[index + 1];
            let p3 = if index + 2 >= points.len() {
                points[points.len() - 1]
            } else {
                points[index + 2]
            };
            let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
            let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
            stroke_path.push_str(&format!(
                " C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}",
                c1.0, c1.1, c2.0, c2.1, p2.0, p2.1
            ));
        }

        let endpoint_x = points.last().map(|point| point.0).unwrap_or(CHART_WIDTH);
        let endpoint_y = points.last().map(|point| point.1).unwrap_or(CHART_BASELINE);
        let area_path = format!(
            "{} L {:.1} {:.1} L 0 {:.1} Z",
            stroke_path, CHART_WIDTH, CHART_BASELINE, CHART_BASELINE
        );
        let chart_y_at = |x: f32| {
            let index = ((x / CHART_WIDTH) * (self.samples.len() - 1) as f32)
                .round()
                .clamp(0.0, (self.samples.len() - 1) as f32) as usize;
            points[index].1
        };

        (
            stroke_path,
            area_path,
            endpoint_x,
            endpoint_y,
            self.current_smoothed_speed > 0.5,
            chart_y_at(65.0),
            chart_y_at(130.0),
            chart_y_at(260.0),
            chart_y_at(320.0),
            chart_y_at(370.0),
        )
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
