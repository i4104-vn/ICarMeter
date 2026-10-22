// MapLibre Native renderer presented in the framebuffer dashboard.

use super::style::MapAppearance;
use crate::models::vehicle::state::VehicleState;
use crate::render::core::FontRenderer;
use maplibre_native::{CameraUpdate, ImageRendererBuilder, LatLng};
use serde_json::Value;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TrySendError};
use std::thread;
use std::time::{Duration, Instant};
use tiny_skia::{
    FillRule, FilterQuality, LineCap, LineJoin, Mask, Paint, PathBuilder, Pixmap, PixmapPaint,
    Stroke, Transform,
};
use url::Url;

pub const VIEW_W: u32 = 382;
pub const VIEW_H: u32 = 396;
const CARD_RADIUS: f32 = 14.0;
const DEFAULT_STYLE_PATH: &str = "/opt/icarmeter/maps/style.json";
const DEFAULT_LATITUDE: f64 = 21.0285;
const DEFAULT_LONGITUDE: f64 = 105.8542;
const MAP_ZOOM: f64 = 17.0;
// Large enough that the viewport remains covered while the cached frame is
// rotated through any heading and panned between 3 m MapLibre renders.
const RENDER_SIZE: u32 = 640;
const RENDER_W: u32 = RENDER_SIZE;
const RENDER_H: u32 = RENDER_SIZE;
const MAP_REFRESH_DISTANCE_M: f64 = 3.0;
const EARTH_RADIUS_M: f64 = 6_371_000.0;

#[derive(Clone, Copy, Debug)]
struct CameraRequest {
    latitude: f64,
    longitude: f64,
    heading: f64,
}

struct MapFrame {
    center: CameraRequest,
    pixels: Vec<u8>,
}

pub struct MapPanel {
    appearance: MapAppearance,
    map_pixmap: Pixmap,
    render_pixmap: Pixmap,
    map_frame: Vec<u8>,
    clip_mask: Mask,
    requests: Option<SyncSender<CameraRequest>>,
    frames: Option<Receiver<MapFrame>>,
    error_rx: Option<Receiver<String>>,
    load_error: Option<String>,
    last_request: Option<CameraRequest>,
    rendered_center: Option<CameraRequest>,
    gps_smoother: GpsPositionSmoother,
    map_worker_stopped: bool,
}

impl MapPanel {
    pub fn new() -> Self {
        let appearance = MapAppearance::default();
        let mut map_pixmap =
            Pixmap::new(VIEW_W, VIEW_H).expect("Failed to allocate MapPanel sub-pixmap");
        map_pixmap.fill(appearance.background_color);
        let render_pixmap =
            Pixmap::new(RENDER_W, RENDER_H).expect("Failed to allocate MapLibre overscan buffer");

        // Pre-compute rounded-rectangle clip mask for zero-cost viewport bounds
        let mut clip_mask = Mask::new(VIEW_W, VIEW_H).expect("Failed to allocate MapPanel mask");
        let mut pb = PathBuilder::new();
        pb.move_to(CARD_RADIUS, 0.0);
        pb.line_to(VIEW_W as f32 - CARD_RADIUS, 0.0);
        pb.quad_to(VIEW_W as f32, 0.0, VIEW_W as f32, CARD_RADIUS);
        pb.line_to(VIEW_W as f32, VIEW_H as f32 - CARD_RADIUS);
        pb.quad_to(
            VIEW_W as f32,
            VIEW_H as f32,
            VIEW_W as f32 - CARD_RADIUS,
            VIEW_H as f32,
        );
        pb.line_to(CARD_RADIUS, VIEW_H as f32);
        pb.quad_to(0.0, VIEW_H as f32, 0.0, VIEW_H as f32 - CARD_RADIUS);
        pb.line_to(0.0, CARD_RADIUS);
        pb.quad_to(0.0, 0.0, CARD_RADIUS, 0.0);
        if let Some(path) = pb.finish() {
            clip_mask.fill_path(&path, FillRule::Winding, true, Transform::identity());
        }

        let mut panel = Self {
            appearance,
            map_frame: render_pixmap.data().to_vec(),
            map_pixmap,
            render_pixmap,
            clip_mask,
            requests: None,
            frames: None,
            error_rx: None,
            load_error: None,
            last_request: None,
            rendered_center: None,
            gps_smoother: GpsPositionSmoother::new(),
            map_worker_stopped: false,
        };

        // Eagerly start MapLibre in its own dedicated background thread at startup
        panel.start_worker();

        panel
    }

    /// Draws the map centered on the current GPS position, or explicit error state if failed.
    pub fn draw(&mut self, pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState) {
        let view_x = 14.0;
        let view_y = 24.0;
        let w = VIEW_W as f32;
        let h = VIEW_H as f32;

        // Check for error notifications from background worker thread
        if let Some(error_rx) = &self.error_rx {
            while let Ok(err) = error_rx.try_recv() {
                self.load_error = Some(err);
                self.map_worker_stopped = true;
            }
        }

        // Keep the completed map frame while the next 3 m update is rendered.
        if let Some(frames) = &self.frames {
            while let Ok(frame) = frames.try_recv() {
                if frame.pixels.len() == self.map_frame.len() {
                    self.map_frame = frame.pixels;
                    self.rendered_center = Some(frame.center);
                }
            }
        }

        let has_gps_position =
            state.gps_connected && valid_coordinates(state.gps_latitude, state.gps_longitude);
        let raw_position = if has_gps_position {
            CameraRequest {
                latitude: state.gps_latitude,
                longitude: state.gps_longitude,
                heading: normalized_heading(state.gps_heading as f64),
            }
        } else {
            CameraRequest {
                latitude: DEFAULT_LATITUDE,
                longitude: DEFAULT_LONGITUDE,
                heading: 0.0,
            }
        };
        let mut camera = self.gps_smoother.update(raw_position, has_gps_position);
        camera.heading = raw_position.heading;
        let refresh_due = self
            .last_request
            .is_none_or(|last| distance_m(last, camera) >= MAP_REFRESH_DISTANCE_M);
        if !self.map_worker_stopped && refresh_due {
            if let Some(requests) = &self.requests {
                match requests.try_send(camera) {
                    Ok(()) => {
                        self.last_request = Some(camera);
                    }
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        eprintln!("[Map] MapLibre renderer thread stopped");
                        self.map_worker_stopped = true;
                        if self.load_error.is_none() {
                            self.load_error = Some("Map renderer thread stopped".to_string());
                        }
                    }
                }
            }
        }

        self.map_pixmap.fill(self.appearance.background_color);

        if let Some(ref error_msg) = self.load_error {
            // Strictly display error screen instead of fallback HUD
            draw_map_error(&mut self.map_pixmap, fonts, w, h, error_msg);
        } else if let Some(center) = self.rendered_center {
            self.render_pixmap
                .data_mut()
                .copy_from_slice(&self.map_frame);
            let (pan_x, pan_y) = map_pan_pixels(center, camera);
            let delta = heading_delta(center.heading, camera.heading).to_radians() as f32;
            let (sin, cos) = delta.sin_cos();
            let center_x = RENDER_W as f32 * 0.5;
            let center_y = RENDER_H as f32 * 0.5;
            let transform = Transform::from_row(
                cos,
                -sin,
                sin,
                cos,
                w * 0.5 + pan_x - cos * center_x - sin * center_y,
                h * 0.5 + pan_y + sin * center_x - cos * center_y,
            );
            let mut paint = PixmapPaint::default();
            paint.quality = FilterQuality::Bilinear;
            self.map_pixmap
                .draw_pixmap(0, 0, self.render_pixmap.as_ref(), &paint, transform, None);

            // Keep the vehicle marker fixed while rotating the map to its heading.
            if has_gps_position {
                draw_vehicle_arrow(&mut self.map_pixmap, w * 0.5, h * 0.5);
            }

            fonts.draw_text(
                &mut self.map_pixmap,
                &fonts.regular,
                "© OpenStreetMap contributors · BBBike",
                8.0,
                h - 7.0,
                7.0,
                tiny_skia::Color::from_rgba8(220, 229, 235, 230),
            );
        } else {
            // Still waiting for initial map frame to complete rendering
            draw_map_loading(&mut self.map_pixmap, fonts, w, h);
        }

        // Clip map, error/loading, position arrow, and attribution to the rounded panel.
        self.map_pixmap.apply_mask(&self.clip_mask);

        let mut paint = PixmapPaint::default();
        paint.opacity = 1.0;
        pixmap.draw_pixmap(
            view_x as i32,
            view_y as i32,
            self.map_pixmap.as_ref(),
            &paint,
            Transform::identity(),
            None,
        );

        self.draw_card_border(pixmap, view_x, view_y, w, h);
    }

    fn start_worker(&mut self) {
        if self.requests.is_some() || self.map_worker_stopped {
            return;
        }

        unsafe {
            std::env::set_var("UV_USE_IO_URING", "0");
        }

        let (request_tx, request_rx) = mpsc::sync_channel::<CameraRequest>(1);
        let (frame_tx, frame_rx) = mpsc::sync_channel::<MapFrame>(1);
        let (error_tx, error_rx) = mpsc::channel::<String>();
        let style_path = std::env::var_os("ICARMETER_MAP_STYLE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_STYLE_PATH));

        let worker_error_tx = error_tx.clone();
        match thread::Builder::new()
            .name("maplibre-renderer".to_string())
            .spawn(move || {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    map_worker(style_path, request_rx, frame_tx, worker_error_tx.clone());
                }));
                if res.is_err() {
                    let _ = worker_error_tx.send("Map renderer worker thread panicked".to_string());
                }
            })
        {
            Ok(_) => {
                self.requests = Some(request_tx);
                self.frames = Some(frame_rx);
                self.error_rx = Some(error_rx);
            }
            Err(error) => {
                eprintln!("[Map] Failed to start MapLibre renderer thread: {error}");
                self.load_error = Some(format!("Failed to spawn thread: {error}"));
                self.map_worker_stopped = true;
            }
        }
    }

    /// Draws outer card border around the map panel
    fn draw_card_border(&self, pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32) {
        let mut pb = PathBuilder::new();
        pb.move_to(x + CARD_RADIUS, y);
        pb.line_to(x + w - CARD_RADIUS, y);
        pb.quad_to(x + w, y, x + w, y + CARD_RADIUS);
        pb.line_to(x + w, y + h - CARD_RADIUS);
        pb.quad_to(x + w, y + h, x + w - CARD_RADIUS, y + h);
        pb.line_to(x + CARD_RADIUS, y + h);
        pb.quad_to(x, y + h, x, y + h - CARD_RADIUS);
        pb.line_to(x, y + CARD_RADIUS);
        pb.quad_to(x, y, x + CARD_RADIUS, y);

        if let Some(path) = pb.finish() {
            let mut border_paint = Paint::default();
            border_paint.set_color(self.appearance.card_border);
            let stroke = Stroke {
                width: 1.2,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
        }
    }
}

fn valid_coordinates(latitude: f64, longitude: f64) -> bool {
    latitude.is_finite()
        && longitude.is_finite()
        && (-90.0..=90.0).contains(&latitude)
        && (-180.0..=180.0).contains(&longitude)
}

// MapLibre Native owns a GL context and its renderer is thread-bound. Keep it
// MapLibre Native owns a GL context and its renderer is thread-bound. Keep it
// on this worker so reading tiles/rendering never stalls the framebuffer loop.
fn map_worker(
    style_path: PathBuf,
    requests: Receiver<CameraRequest>,
    frames: SyncSender<MapFrame>,
    errors: Sender<String>,
) {
    let canonical_path = match std::fs::canonicalize(&style_path) {
        Ok(path) => path,
        Err(error) => {
            let msg = format!("Style not found: {}", style_path.display());
            eprintln!(
                "[Map] Cannot resolve offline MapLibre style '{}': {error}",
                style_path.display()
            );
            let _ = errors.send(msg);
            return;
        }
    };
    let style_json = match std::fs::read_to_string(&canonical_path) {
        Ok(json) => json,
        Err(error) => {
            let msg = format!("Cannot read style file: {error}");
            eprintln!(
                "[Map] Cannot read offline MapLibre style '{}': {error}",
                canonical_path.display()
            );
            let _ = errors.send(msg);
            return;
        }
    };
    let style_value: Value = match serde_json::from_str(&style_json) {
        Ok(value) => value,
        Err(error) => {
            let msg = format!("Invalid style JSON: {error}");
            eprintln!("[Map] Invalid offline MapLibre style JSON: {error}");
            let _ = errors.send(msg);
            return;
        }
    };
    if let Err(error) = validate_offline_style(&style_value) {
        let msg = format!("Style validation failed: {error}");
        eprintln!("[Map] Offline MapLibre style rejected: {error}");
        let _ = errors.send(msg);
        return;
    }
    let style_uri = match Url::from_file_path(&canonical_path) {
        Ok(uri) => uri,
        Err(()) => {
            let msg = "Invalid file path URL for style".to_string();
            eprintln!("[Map] Offline MapLibre style path is not a valid file path");
            let _ = errors.send(msg);
            return;
        }
    };

    let mut renderer = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ImageRendererBuilder::new()
            .with_size(
                NonZeroU32::new(RENDER_W).unwrap(),
                NonZeroU32::new(RENDER_H).unwrap(),
            )
            .build_static_renderer()
    })) {
        Ok(renderer) => renderer,
        Err(err) => {
            let msg = "Failed to initialize MapLibre static renderer".to_string();
            eprintln!("[Map] Panic while creating MapLibre static renderer: {:?}", err);
            let _ = errors.send(msg);
            return;
        }
    };

    let load_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        renderer.load_style_from_url(&style_uri).wait()
    }));

    match load_res {
        Ok(Ok(())) => {
            eprintln!("[Map] Offline MapLibre style loaded successfully");
        }
        Ok(Err(error)) => {
            let msg = format!("Failed to load style: {error}");
            eprintln!("[Map] Failed to load offline MapLibre style: {error}");
            let _ = errors.send(msg);
            return;
        }
        Err(panic_err) => {
            let msg = "Panic while loading MapLibre style".to_string();
            eprintln!("[Map] Panic while loading MapLibre style: {:?}", panic_err);
            let _ = errors.send(msg);
            return;
        }
    }

    while let Ok(mut camera) = requests.recv() {
        while let Ok(newer) = requests.try_recv() {
            camera = newer;
        }
        let camera_update = CameraUpdate::new()
            .center(LatLng {
                lat: camera.latitude,
                lng: camera.longitude,
            })
            .zoom(MAP_ZOOM)
            .bearing(camera.heading);
        let render_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            renderer.render_static(&camera_update)
        }));
        let image = match render_res {
            Ok(Ok(image)) => image,
            Ok(Err(error)) => {
                eprintln!("[Map] MapLibre snapshot failed: {error}");
                continue;
            }
            Err(panic_err) => {
                eprintln!("[Map] Panic during MapLibre render_static: {:?}", panic_err);
                continue;
            }
        };
        match frames.try_send(MapFrame {
            center: camera,
            pixels: image.as_image().as_raw().to_vec(),
        }) {
            Ok(()) | Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => return,
        }
    }
}

struct GpsPositionSmoother {
    last_raw: Option<CameraRequest>,
    from: CameraRequest,
    target: CameraRequest,
    started: Instant,
    duration: Duration,
    last_sample_at: Option<Instant>,
}

impl GpsPositionSmoother {
    fn new() -> Self {
        let initial = CameraRequest {
            latitude: DEFAULT_LATITUDE,
            longitude: DEFAULT_LONGITUDE,
            heading: 0.0,
        };
        Self {
            last_raw: None,
            from: initial,
            target: initial,
            started: Instant::now(),
            duration: Duration::ZERO,
            last_sample_at: None,
        }
    }

    fn update(&mut self, raw: CameraRequest, has_gps_position: bool) -> CameraRequest {
        let now = Instant::now();
        if !has_gps_position {
            self.last_raw = Some(raw);
            self.from = raw;
            self.target = raw;
            self.started = now;
            self.duration = Duration::ZERO;
            self.last_sample_at = Some(now);
            return raw;
        }

        let sample_changed = self
            .last_raw
            .is_none_or(|last| distance_m(last, raw) >= 0.75);
        if sample_changed {
            let current = self.position_at(now);
            let sample_interval = self
                .last_sample_at
                .map(|last| now.saturating_duration_since(last))
                .unwrap_or(Duration::from_millis(50));
            self.from = current;
            self.target = raw;
            self.started = now;
            self.duration = sample_interval
                .max(Duration::from_millis(50))
                .min(Duration::from_secs(1));
            self.last_raw = Some(raw);
            self.last_sample_at = Some(now);
        }
        self.position_at(now)
    }

    fn position_at(&self, now: Instant) -> CameraRequest {
        if self.duration.is_zero() {
            return self.target;
        }
        let progress = (now.saturating_duration_since(self.started).as_secs_f64()
            / self.duration.as_secs_f64())
        .clamp(0.0, 1.0);
        interpolate_position(self.from, self.target, progress)
    }
}

fn interpolate_position(from: CameraRequest, to: CameraRequest, progress: f64) -> CameraRequest {
    let longitude_delta = (to.longitude - from.longitude + 180.0).rem_euclid(360.0) - 180.0;
    CameraRequest {
        latitude: from.latitude + (to.latitude - from.latitude) * progress,
        longitude: (from.longitude + longitude_delta * progress + 180.0).rem_euclid(360.0) - 180.0,
        heading: from.heading,
    }
}

fn distance_m(from: CameraRequest, to: CameraRequest) -> f64 {
    let latitude_delta = (to.latitude - from.latitude).to_radians();
    let longitude_delta = (to.longitude - from.longitude).to_radians();
    let from_latitude = from.latitude.to_radians();
    let to_latitude = to.latitude.to_radians();
    let haversine = (latitude_delta * 0.5).sin().powi(2)
        + from_latitude.cos() * to_latitude.cos() * (longitude_delta * 0.5).sin().powi(2);
    EARTH_RADIUS_M * 2.0 * haversine.sqrt().atan2((1.0 - haversine).sqrt())
}

fn map_pan_pixels(rendered: CameraRequest, current: CameraRequest) -> (f32, f32) {
    let latitude = ((rendered.latitude + current.latitude) * 0.5).to_radians();
    let east_m =
        (current.longitude - rendered.longitude).to_radians() * EARTH_RADIUS_M * latitude.cos();
    let north_m = (current.latitude - rendered.latitude).to_radians() * EARTH_RADIUS_M;
    let meters_per_pixel = (2.0 * std::f64::consts::PI * EARTH_RADIUS_M * latitude.cos())
        / (256.0 * 2.0_f64.powf(MAP_ZOOM));
    // Pan in the current screen orientation because rotation is applied around
    // the rendered frame's center before this translation.
    let (sin, cos) = current.heading.to_radians().sin_cos();
    let east_px = east_m / meters_per_pixel;
    let north_px = north_m / meters_per_pixel;
    (
        (-cos * east_px + sin * north_px) as f32,
        (sin * east_px + cos * north_px) as f32,
    )
}

fn normalized_heading(heading: f64) -> f64 {
    if heading.is_finite() {
        heading.rem_euclid(360.0)
    } else {
        0.0
    }
}

fn heading_delta(from: f64, to: f64) -> f64 {
    (to - from + 180.0).rem_euclid(360.0) - 180.0
}

fn draw_vehicle_arrow(pixmap: &mut Pixmap, center_x: f32, center_y: f32) {
    let forward_x = 0.0;
    let forward_y = -1.0;
    let side_x = 1.0;
    let side_y = 0.0;
    let mut arrow = PathBuilder::new();
    arrow.move_to(center_x + forward_x * 12.0, center_y + forward_y * 12.0);
    arrow.line_to(
        center_x - forward_x * 7.0 + side_x * 6.0,
        center_y - forward_y * 7.0 + side_y * 6.0,
    );
    arrow.line_to(center_x - forward_x * 4.0, center_y - forward_y * 4.0);
    arrow.line_to(
        center_x - forward_x * 7.0 - side_x * 6.0,
        center_y - forward_y * 7.0 - side_y * 6.0,
    );
    arrow.close();

    if let Some(path) = arrow.finish() {
        let mut fill = Paint::default();
        fill.set_color(tiny_skia::Color::from_rgba8(56, 189, 248, 255));
        pixmap.fill_path(&path, &fill, FillRule::Winding, Transform::identity(), None);
        let mut outline = Paint::default();
        outline.set_color(tiny_skia::Color::WHITE);
        pixmap.stroke_path(
            &path,
            &outline,
            &Stroke {
                width: 1.2,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );
    }
}

fn wrap_text(
    fonts: &FontRenderer,
    text: &str,
    max_width: f32,
    font_size: f32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{current_line} {word}")
        };
        if fonts.measure_width(&fonts.regular, &test_line, font_size) <= max_width {
            current_line = test_line;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.len() > 3 {
        lines.truncate(3);
        if let Some(last) = lines.last_mut() {
            last.push_str("...");
        }
    }
    lines
}

fn draw_map_error(
    pixmap: &mut Pixmap,
    fonts: &FontRenderer,
    w: f32,
    h: f32,
    error_msg: &str,
) {
    let center_x = w * 0.5;
    let center_y = h * 0.40;

    // 1. Warning badge container
    let box_w = 46.0;
    let box_h = 46.0;
    let box_x = center_x - box_w * 0.5;
    let box_y = center_y - 64.0;

    let rounded_box = crate::render::core::create_rounded_rect(box_x, box_y, box_w, box_h, 12.0);
    let mut box_paint = Paint::default();
    box_paint.set_color(tiny_skia::Color::from_rgba8(239, 68, 68, 28));
    box_paint.anti_alias = true;
    pixmap.fill_path(&rounded_box, &box_paint, FillRule::Winding, Transform::identity(), None);

    let mut box_border = Paint::default();
    box_border.set_color(tiny_skia::Color::from_rgba8(239, 68, 68, 140));
    box_border.anti_alias = true;
    let box_stroke = Stroke {
        width: 1.2,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    };
    pixmap.stroke_path(&rounded_box, &box_border, &box_stroke, Transform::identity(), None);

    // 2. Warning Triangle Icon inside badge
    let icon_cy = box_y + box_h * 0.5;
    let mut triangle = PathBuilder::new();
    triangle.move_to(center_x, icon_cy - 10.0);
    triangle.line_to(center_x + 11.0, icon_cy + 9.0);
    triangle.line_to(center_x - 11.0, icon_cy + 9.0);
    triangle.close();
    if let Some(path) = triangle.finish() {
        let mut tri_paint = Paint::default();
        tri_paint.set_color(tiny_skia::Color::from_rgba8(248, 113, 113, 255));
        tri_paint.anti_alias = true;
        let tri_stroke = Stroke {
            width: 1.6,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &tri_paint, &tri_stroke, Transform::identity(), None);
    }

    // Exclamation mark inside triangle
    let mut excl = PathBuilder::new();
    excl.move_to(center_x, icon_cy - 4.0);
    excl.line_to(center_x, icon_cy + 2.5);
    if let Some(path) = excl.finish() {
        let mut excl_paint = Paint::default();
        excl_paint.set_color(tiny_skia::Color::from_rgba8(248, 113, 113, 255));
        excl_paint.anti_alias = true;
        let excl_stroke = Stroke {
            width: 1.6,
            line_cap: LineCap::Round,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &excl_paint, &excl_stroke, Transform::identity(), None);
    }
    // Dot of exclamation mark
    let mut dot = PathBuilder::new();
    dot.push_circle(center_x, icon_cy + 6.0, 0.9);
    if let Some(path) = dot.finish() {
        let mut dot_paint = Paint::default();
        dot_paint.set_color(tiny_skia::Color::from_rgba8(248, 113, 113, 255));
        dot_paint.anti_alias = true;
        pixmap.fill_path(&path, &dot_paint, FillRule::Winding, Transform::identity(), None);
    }

    // 3. Category header: "OFFLINE MAP"
    let cat_text = "OFFLINE MAP";
    let cat_w = fonts.measure_width(&fonts.bold, cat_text, 10.0);
    fonts.draw_text(
        pixmap,
        &fonts.bold,
        cat_text,
        center_x - cat_w * 0.5,
        center_y + 8.0,
        10.0,
        tiny_skia::Color::from_rgba8(148, 163, 184, 180),
    );

    // 4. Primary Error title: "Failed to load"
    let title_text = "Failed to load";
    let title_w = fonts.measure_width(&fonts.bold, title_text, 19.0);
    fonts.draw_text(
        pixmap,
        &fonts.bold,
        title_text,
        center_x - title_w * 0.5,
        center_y + 32.0,
        19.0,
        tiny_skia::Color::from_rgba8(248, 113, 113, 255),
    );

    // 5. Details container box with wrapped message
    let lines = wrap_text(fonts, error_msg, w - 48.0, 9.5);
    let line_h = 14.0;
    let detail_box_h = (lines.len() as f32 * line_h + 16.0).max(36.0);
    let detail_box_w = w - 36.0;
    let detail_box_x = 18.0;
    let detail_box_y = center_y + 46.0;

    let detail_rect = crate::render::core::create_rounded_rect(
        detail_box_x,
        detail_box_y,
        detail_box_w,
        detail_box_h,
        8.0,
    );
    let mut bg_paint = Paint::default();
    bg_paint.set_color(tiny_skia::Color::from_rgba8(15, 23, 42, 220));
    bg_paint.anti_alias = true;
    pixmap.fill_path(&detail_rect, &bg_paint, FillRule::Winding, Transform::identity(), None);

    let mut border_paint = Paint::default();
    border_paint.set_color(tiny_skia::Color::from_rgba8(51, 65, 85, 140));
    border_paint.anti_alias = true;
    let border_stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(&detail_rect, &border_paint, &border_stroke, Transform::identity(), None);

    let start_text_y = detail_box_y + 14.0 + (detail_box_h - (lines.len() as f32 * line_h + 10.0)) * 0.5;
    for (idx, line) in lines.iter().enumerate() {
        let line_w = fonts.measure_width(&fonts.regular, line, 9.5);
        let text_x = center_x - line_w * 0.5;
        let text_y = start_text_y + idx as f32 * line_h;
        fonts.draw_text(
            pixmap,
            &fonts.regular,
            line,
            text_x,
            text_y,
            9.5,
            tiny_skia::Color::from_rgba8(203, 213, 225, 230),
        );
    }

    // 6. Footer toggle hint: "[M] Toggle TPMS / Map"
    let hint_text = "[M] Toggle TPMS / Map";
    let hint_w = fonts.measure_width(&fonts.regular, hint_text, 9.5);
    fonts.draw_text(
        pixmap,
        &fonts.regular,
        hint_text,
        center_x - hint_w * 0.5,
        h - 18.0,
        9.5,
        tiny_skia::Color::from_rgba8(148, 163, 184, 180),
    );
}

fn draw_map_loading(pixmap: &mut Pixmap, fonts: &FontRenderer, w: f32, h: f32) {
    let center_x = w * 0.5;
    let center_y = h * 0.44;

    // Header badge
    let header_text = "OFFLINE MAP";
    let header_w = fonts.measure_width(&fonts.bold, header_text, 10.0);
    fonts.draw_text(
        pixmap,
        &fonts.bold,
        header_text,
        center_x - header_w * 0.5,
        center_y - 20.0,
        10.0,
        tiny_skia::Color::from_rgba8(56, 189, 248, 220),
    );

    // Title
    let title_text = "Loading map...";
    let title_w = fonts.measure_width(&fonts.bold, title_text, 17.0);
    fonts.draw_text(
        pixmap,
        &fonts.bold,
        title_text,
        center_x - title_w * 0.5,
        center_y + 8.0,
        17.0,
        tiny_skia::Color::from_rgba8(226, 232, 240, 240),
    );

    let sub_text = "Initializing MapLibre vector renderer";
    let sub_w = fonts.measure_width(&fonts.regular, sub_text, 10.0);
    fonts.draw_text(
        pixmap,
        &fonts.regular,
        sub_text,
        center_x - sub_w * 0.5,
        center_y + 26.0,
        10.0,
        tiny_skia::Color::from_rgba8(148, 163, 184, 180),
    );

    // Toggle hint
    let hint_text = "[M] Toggle TPMS / Map";
    let hint_w = fonts.measure_width(&fonts.regular, hint_text, 9.5);
    fonts.draw_text(
        pixmap,
        &fonts.regular,
        hint_text,
        center_x - hint_w * 0.5,
        h - 18.0,
        9.5,
        tiny_skia::Color::from_rgba8(148, 163, 184, 180),
    );
}

fn validate_offline_style(style: &Value) -> Result<(), String> {
    fn check_resource_urls(value: &Value, key: Option<&str>) -> Result<(), String> {
        match value {
            Value::Object(object) => {
                for (child_key, child_value) in object {
                    check_resource_urls(child_value, Some(child_key))?;
                }
            }
            Value::Array(items) => {
                for item in items {
                    check_resource_urls(item, key)?;
                }
            }
            Value::String(resource)
                if matches!(key, Some("url" | "tiles" | "glyphs" | "sprite"))
                    && (resource.starts_with("http://")
                        || resource.starts_with("https://")
                        || resource.starts_with("mapbox://")) =>
            {
                return Err(format!(
                    "resource '{resource}' is remote; use local files or MBTiles"
                ));
            }
            Value::String(resource) if key == Some("url") && resource.starts_with("mbtiles://") => {
                let uri = Url::parse(resource)
                    .map_err(|error| format!("invalid MBTiles URL '{resource}': {error}"))?;
                let path = Path::new(uri.path());
                if !path.is_file() {
                    return Err(format!(
                        "MBTiles resource '{}' does not exist",
                        path.display()
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    check_resource_urls(style, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Color;

    #[test]
    fn test_render_map_panel() {
        let mut pixmap = Pixmap::new(410, 480).unwrap();
        pixmap.fill(Color::BLACK);
        let mut state = VehicleState::default();
        state.gps_connected = true;
        state.gps_latitude = 21.0285;
        state.gps_longitude = 105.8542;
        let mut panel = MapPanel::new();
        let fonts = FontRenderer::new().unwrap();
        panel.draw(&mut pixmap, &fonts, &state);
    }

    #[test]
    fn test_render_dashboard_with_map() {
        use crate::render::DashboardRenderer;
        let mut renderer = DashboardRenderer::new().unwrap();
        let mut state = VehicleState::default();
        state.tpms_enabled = false;
        state.current_speed = 65.0;
        state.gps_connected = true;
        let _ = renderer.render(&state, 2.0);
        let _ = renderer.pixmap().save_png("/home/i4104/.gemini/antigravity-ide/brain/3c4551f7-044f-4cc0-bc1b-9bf1ceb3095b/dashboard_map_preview.png");
    }

    #[test]
    fn test_render_map_panel_error_state() {
        let mut pixmap = Pixmap::new(410, 480).unwrap();
        pixmap.fill(Color::BLACK);
        let state = VehicleState::default();
        let mut panel = MapPanel::new();
        panel.load_error = Some("Style not found: /opt/icarmeter/maps/style.json".to_string());
        let fonts = FontRenderer::new().unwrap();
        panel.draw(&mut pixmap, &fonts, &state);
        let _ = pixmap.save_png("/home/i4104/.gemini/antigravity-ide/brain/3c4551f7-044f-4cc0-bc1b-9bf1ceb3095b/map_error_preview.png");
    }
}
