// MapLibre GL 2D Offline Navigation Renderer
// Implements Heading-Up track rotation (arrow always points UP at 0°)
// and modular layer rendering adhering to MapLibre Style Specification v8.

use super::arrow::VehicleArrow;
use super::camera::Camera;
use super::hud::HudOverlay;
use super::layers::{BuildingLayer, GridLayer, RoadLayer, WaterLayer, WaypointLayer};
use super::style::MapLibreStyle;
use crate::models::VehicleState;
use crate::render::core::FontRenderer;
use tiny_skia::{
    FillRule, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform,
};

pub const VIEW_W: u32 = 382;
pub const VIEW_H: u32 = 396;
const CARD_RADIUS: f32 = 14.0;

pub struct MapPanel {
    pub style: MapLibreStyle,
    map_pixmap: Pixmap,
    clip_mask: Mask,
    sim_distance: f32,
    pulse_phase: f32,
}

impl MapPanel {
    /// Creates a new MapPanel with the MapLibre Dark Navigation style
    pub fn new() -> Self {
        Self::with_style(MapLibreStyle::dark_navigation())
    }

    /// Creates a new MapPanel with a custom MapLibre style
    pub fn with_style(style: MapLibreStyle) -> Self {
        let mut map_pixmap =
            Pixmap::new(VIEW_W, VIEW_H).expect("Failed to allocate MapPanel sub-pixmap");
        map_pixmap.fill(style.background_color);

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
            style,
            map_pixmap,
            clip_mask,
            sim_distance: 0.0,
            pulse_phase: 0.0,
        }
    }

    /// Renders the complete 2D MapLibre navigation panel into the target pixmap
    pub fn draw(
        &mut self,
        pixmap: &mut Pixmap,
        fonts: &FontRenderer,
        state: &VehicleState,
        dt: f32,
    ) {
        let view_x = 14.0;
        let view_y = 24.0;
        let w = VIEW_W as f32;
        let h = VIEW_H as f32;

        self.pulse_phase = (self.pulse_phase + dt * 2.2) % (std::f32::consts::PI * 2.0);

        // 1. Clear sub-pixmap with MapLibre tactical background color
        self.map_pixmap.fill(self.style.background_color);

        // 2. Fixed screen anchor for the vehicle (center X, forward visibility on Y)
        let anchor_x = w * 0.5;  // 191.0
        let anchor_y = h * 0.62; // 245.5

        // Advance simulation distance when no GPS speed is reported
        let effective_speed = match state.current_speed > 0.0 {
            true => state.current_speed,
            false => match state.gps_connected {
                true => 0.0,
                false => 36.0, // Cruise speed for offline demo
            },
        };
        self.sim_distance += effective_speed * (dt / 3.6) * 2.2;

        // Current heading in degrees (0 = North, 90 = East, 180 = South, 270 = West)
        let heading_deg = match state.gps_heading > 0.0 {
            true => state.gps_heading,
            false => {
                let turn = (self.sim_distance * 0.0018).sin() * 50.0;
                (325.0 + turn).rem_euclid(360.0)
            }
        };

        // 3. World map coordinates of vehicle
        let car_map_x = (self.sim_distance * 0.0012).sin() * 40.0;
        let car_map_y = -self.sim_distance;

        // 4. MapLibre Camera & Heading-Up Matrix Transformation
        let camera = Camera::new(car_map_x, car_map_y, heading_deg, anchor_x, anchor_y);
        let map_transform = camera.compute_transform();

        // 5. Render MapLibre vector layers in specification order
        WaterLayer::draw(&mut self.map_pixmap, &self.style, map_transform);
        BuildingLayer::draw(&mut self.map_pixmap, &self.style, map_transform, car_map_y);
        GridLayer::draw(&mut self.map_pixmap, &self.style, map_transform, car_map_y);
        RoadLayer::draw(&mut self.map_pixmap, &self.style, map_transform, car_map_y);
        WaypointLayer::draw(&mut self.map_pixmap, &self.style, map_transform, car_map_y);

        // 6. Navigation Vehicle Arrow (PERMANENTLY POINTING STRAIGHT UP: 0°)
        VehicleArrow::draw(
            &mut self.map_pixmap,
            &self.style,
            anchor_x,
            anchor_y,
            self.pulse_phase,
        );

        // 7. Tactical HUD Overlays (Compass, Coordinates, Fix Badge, Scale)
        HudOverlay::draw(
            &mut self.map_pixmap,
            fonts,
            &self.style,
            w,
            h,
            heading_deg,
            state,
        );

        // 8. Apply Rounded Mask to clip all rotated vector features
        self.map_pixmap.apply_mask(&self.clip_mask);

        // 9. Blit clipped map into main dashboard pixmap
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

        // 10. Outer card border
        self.draw_card_border(pixmap, view_x, view_y, w, h);
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
            border_paint.set_color(self.style.card_border);
            let stroke = Stroke {
                width: 1.2,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Color;

    #[test]
    fn test_render_map_panel() {
        let mut pixmap = Pixmap::new(410, 480).unwrap();
        pixmap.fill(Color::BLACK);
        let fonts = FontRenderer::new().unwrap();
        let mut state = VehicleState::default();
        state.gps_connected = true;
        state.gps_heading = 45.0;
        state.gps_latitude = 21.0285;
        state.gps_longitude = 105.8542;
        let mut panel = MapPanel::new();
        panel.draw(&mut pixmap, &fonts, &state, 0.016);
    }

    #[test]
    fn test_render_dashboard_with_map() {
        use crate::render::DashboardRenderer;
        let mut renderer = DashboardRenderer::new().unwrap();
        let mut state = VehicleState::default();
        state.tpms_enabled = false;
        state.current_speed = 65.0;
        state.gps_connected = true;
        state.gps_heading = 315.0;
        let _ = renderer.render(&state, 2.0);
        let _ = renderer.pixmap().save_png("/home/i4104/.gemini/antigravity-ide/brain/1653a5b5-5bed-4b97-97c8-db6cbffce294/dashboard_map_preview.png");
    }
}
