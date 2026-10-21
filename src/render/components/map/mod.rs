// 2D Offline Tactical Map component built with MapLibre GL specification
// Features Heading-Up track rotation (arrow always points UP at 0°) and smooth coordinate translation.

pub mod arrow;
pub mod camera;
pub mod hud;
pub mod layers;
pub mod renderer;
pub mod style;

pub use renderer::MapPanel;
