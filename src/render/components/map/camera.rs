// MapLibre GL Navigation Camera and Heading-Up Coordinate Transformation
// In Navigation Mode, the camera tracks the vehicle with:
// - Camera center at vehicle position (center_x, center_y)
// - Camera bearing matching vehicle heading
// - Screen vehicle anchor fixed (facing UP at 0°)
// - Map coordinate space rotated by -bearing around the anchor

use tiny_skia::Transform;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub center_x: f32,
    pub center_y: f32,
    pub bearing_deg: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,
}

impl Camera {
    pub fn new(
        center_x: f32,
        center_y: f32,
        bearing_deg: f32,
        anchor_x: f32,
        anchor_y: f32,
    ) -> Self {
        Self {
            center_x,
            center_y,
            bearing_deg,
            anchor_x,
            anchor_y,
        }
    }

    /// Computes the MapLibre Heading-Up Affine Matrix:
    /// Maps world map coordinates (X, Y) to screen viewport (x, y):
    /// P_screen = Anchor + Rot(-bearing) * (P_world - Center)
    pub fn compute_transform(&self) -> Transform {
        let rad = (-self.bearing_deg).to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        // Translation offsets to place Center at Anchor after rotation
        let tx = self.anchor_x - cos_a * self.center_x + sin_a * self.center_y;
        let ty = self.anchor_y - sin_a * self.center_x - cos_a * self.center_y;

        Transform::from_row(cos_a, sin_a, -sin_a, cos_a, tx, ty)
    }
}
