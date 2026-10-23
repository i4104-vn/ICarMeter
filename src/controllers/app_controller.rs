// Main application controller for Slint and the telemetry services.

use crate::configs::app::AppConfig;
use crate::controllers::demo::DemoGps;
use crate::models::bluetooth::BluetoothSharedState;
use crate::models::vehicle::VehicleState;
use crate::services::{TelemetryService, init_signal_handlers, is_app_running};
use slint::{ComponentHandle, ModelRc, Timer, TimerMode, VecModel};
use slint_mapping::TileSource;
use slint_mapping::sources::FileTileSource;
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
        let map = MapRuntime::new();
        ui.set_map_tiles(ModelRc::from(map.tiles.clone()));
        ui.set_map_following(map.follow_gps);
        refresh_map(&ui, &map);
        let runtime = Rc::new(RefCell::new(RuntimeState {
            state,
            demo_gps,
            telemetry,
            map,
            config,
            last_time: Instant::now(),
            last_config_check: Instant::now(),
            last_config_mtime: AppConfig::last_modified(),
            last_map_update: None,
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
        ui.on_map_pan({
            let ui = ui.as_weak();
            let runtime = runtime.clone();
            move |_dx, _dy| {
                // Map is permanently GPS locked; pan does not disconnect follow
                let Some(ui) = ui.upgrade() else { return };
                let runtime = runtime.borrow();
                refresh_map(&ui, &runtime.map);
            }
        });
        ui.on_map_zoom_by({
            let ui = ui.as_weak();
            let runtime = runtime.clone();
            move |_delta, _anchor_x, _anchor_y| {
                // Fixed zoom level - ignore zoom gestures
                let Some(ui) = ui.upgrade() else { return };
                let mut runtime = runtime.borrow_mut();
                runtime.map.zoom = FIXED_MAP_ZOOM;
                refresh_map(&ui, &runtime.map);
            }
        });
        ui.on_map_follow({
            let ui = ui.as_weak();
            let runtime = runtime.clone();
            move || {
                let Some(ui) = ui.upgrade() else { return };
                let mut runtime = runtime.borrow_mut();
                runtime.map.follow_gps = true;
                runtime.map.zoom = FIXED_MAP_ZOOM;
                if runtime.state.gps_connected {
                    runtime.map.latitude = runtime.state.gps_latitude;
                    runtime.map.longitude = runtime.state.gps_longitude;
                }
                ui.set_map_following(true);
                refresh_map(&ui, &runtime.map);
                runtime.last_map_update = Some(Instant::now());
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
                if should_refresh_map(&runtime) {
                    if runtime.state.gps_connected {
                        runtime.map.latitude = runtime.state.gps_latitude;
                        runtime.map.longitude = runtime.state.gps_longitude;
                    }
                    runtime.map.zoom = FIXED_MAP_ZOOM;
                    refresh_map(&ui, &runtime.map);
                    runtime.last_map_update = Some(Instant::now());
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
    map: MapRuntime,
    config: AppConfig,
    last_time: Instant,
    last_config_check: Instant,
    last_config_mtime: Option<std::time::SystemTime>,
    last_map_update: Option<Instant>,
}

// Keep these in sync with MapEmbed's 382×396 rectangle in tpmspanel.slint.
// Keep these in sync with MapEmbed's 382×396 rectangle in tpmspanel.slint.
const MAP_VIEWPORT_WIDTH: f32 = 382.0;
const MAP_VIEWPORT_HEIGHT: f32 = 396.0;
const MAP_FOLLOW_INTERVAL: Duration = Duration::from_millis(200);
const FIXED_MAP_ZOOM: f64 = 14.0;

struct MapRuntime {
    source: FileTileSource,
    tiles: Rc<VecModel<Tile>>,
    latitude: f64,
    longitude: f64,
    zoom: f64,
    follow_gps: bool,
}

const MAP_TILES_DIR: &str = "/opt/icarmeter/maps/tiles";

impl MapRuntime {
    fn new() -> Self {
        let source = FileTileSource::new(MAP_TILES_DIR)
            .with_extension("png")
            .with_zoom_range(0, 16);
        Self {
            source,
            tiles: Rc::new(VecModel::from(Vec::new())),
            latitude: 21.0285,
            longitude: 105.8542,
            zoom: FIXED_MAP_ZOOM,
            follow_gps: true,
        }
    }
}

fn refresh_map(ui: &AppWindow, map: &MapRuntime) {
    let visible = slint_mapping::viewport::visible_tiles(
        map.longitude,
        map.latitude,
        map.zoom,
        f64::from(MAP_VIEWPORT_WIDTH),
        f64::from(MAP_VIEWPORT_HEIGHT),
        map.source.tile_size(),
    );
    let tiles: Vec<Tile> = visible
        .into_iter()
        .filter_map(|placed| {
            map.source.tile(placed.key).map(|image| Tile {
                x: placed.x,
                y: placed.y,
                size: placed.size,
                image,
            })
        })
        .collect();
    ui.set_map_tiles_available(!tiles.is_empty());
    map.tiles.set_vec(tiles);
    ui.set_map_latitude(map.latitude as f32);
    ui.set_map_longitude(map.longitude as f32);
    ui.set_map_zoom(map.zoom as f32);
}

fn should_refresh_map(runtime: &RuntimeState) -> bool {
    let Some(last_update) = runtime.last_map_update else {
        return true;
    };
    if last_update.elapsed() < MAP_FOLLOW_INTERVAL {
        return false;
    }
    if runtime.state.gps_connected {
        let latitude_delta = (runtime.state.gps_latitude - runtime.map.latitude) * 111_320.0;
        let longitude_delta = (runtime.state.gps_longitude - runtime.map.longitude)
            * 111_320.0
            * runtime.map.latitude.to_radians().cos();
        latitude_delta.hypot(longitude_delta) >= 1.0 || (runtime.map.zoom - FIXED_MAP_ZOOM).abs() > 0.01
    } else {
        false
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
