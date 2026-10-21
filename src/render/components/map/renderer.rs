// MapLibre Native renderer presented in the framebuffer dashboard.

use super::style::MapAppearance;
use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use maplibre_native::{CameraUpdate, ImageRendererBuilder, LatLng};
use serde_json::Value;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread;
use std::time::{Duration, Instant};
use tiny_skia::{
    FillRule, FilterQuality, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform,
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

        Self {
            appearance,
            map_frame: render_pixmap.data().to_vec(),
            map_pixmap,
            render_pixmap,
            clip_mask,
            requests: None,
            frames: None,
            last_request: None,
            rendered_center: None,
            gps_smoother: GpsPositionSmoother::new(),
            map_worker_stopped: false,
        }
    }

    /// Draws the map centered on the current GPS position.
    pub fn draw(&mut self, pixmap: &mut Pixmap, fonts: &FontRenderer, state: &VehicleState) {
        let view_x = 14.0;
        let view_y = 24.0;
        let w = VIEW_W as f32;
        let h = VIEW_H as f32;

        self.start_worker();

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
            match self
                .requests
                .as_ref()
                .expect("MapLibre worker is started before drawing")
                .try_send(camera)
            {
                Ok(()) => {
                    self.last_request = Some(camera);
                }
                Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => {
                    eprintln!("MapLibre renderer thread stopped");
                    self.map_worker_stopped = true;
                }
            }
        }

        self.map_pixmap.fill(self.appearance.background_color);
        self.render_pixmap
            .data_mut()
            .copy_from_slice(&self.map_frame);
        if let Some(center) = self.rendered_center {
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
        }

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

        // Clip map, position arrow, and attribution to the rounded panel.
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

        let (request_tx, request_rx) = mpsc::sync_channel::<CameraRequest>(1);
        let (frame_tx, frame_rx) = mpsc::sync_channel::<MapFrame>(1);
        let style_path = std::env::var_os("ICARMETER_MAP_STYLE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_STYLE_PATH));
        match thread::Builder::new()
            .name("maplibre-renderer".to_string())
            .spawn(move || map_worker(style_path, request_rx, frame_tx))
        {
            Ok(_) => {
                self.requests = Some(request_tx);
                self.frames = Some(frame_rx);
            }
            Err(error) => {
                eprintln!("Failed to start MapLibre renderer thread: {error}");
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
// on this worker so reading tiles/rendering never stalls the framebuffer loop.
fn map_worker(
    style_path: PathBuf,
    requests: Receiver<CameraRequest>,
    frames: SyncSender<MapFrame>,
) {
    let style_path = match std::fs::canonicalize(&style_path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!(
                "Cannot resolve offline MapLibre style '{}': {error}",
                style_path.display()
            );
            return;
        }
    };
    let style_json = match std::fs::read_to_string(&style_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!(
                "Cannot read offline MapLibre style '{}': {error}",
                style_path.display()
            );
            return;
        }
    };
    let style_value: Value = match serde_json::from_str(&style_json) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Invalid offline MapLibre style JSON: {error}");
            return;
        }
    };
    if let Err(error) = validate_offline_style(&style_value) {
        eprintln!("Offline MapLibre style rejected: {error}");
        return;
    }
    let style_uri = match Url::from_file_path(&style_path) {
        Ok(uri) => uri,
        Err(()) => {
            eprintln!("Offline MapLibre style path is not a valid file path");
            return;
        }
    };

    // This worker produces one complete snapshot at a time. Validate the
    // style input before creating MapLibre's native graphics context; a
    // missing/invalid style must degrade to the panel background, not take
    // down the framebuffer process during renderer startup.
    let mut renderer = ImageRendererBuilder::new()
        .with_size(
            NonZeroU32::new(RENDER_W).unwrap(),
            NonZeroU32::new(RENDER_H).unwrap(),
        )
        .build_static_renderer();
    if let Err(error) = renderer.load_style_from_url(&style_uri).wait() {
        eprintln!("Failed to load offline MapLibre style: {error}");
        return;
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
        let image = match renderer.render_static(&camera_update) {
            Ok(image) => image,
            Err(error) => {
                eprintln!("MapLibre snapshot failed: {error}");
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
        let _ = renderer.pixmap().save_png("/home/i4104/.gemini/antigravity-ide/brain/1653a5b5-5bed-4b97-97c8-db6cbffce294/dashboard_map_preview.png");
    }
}
