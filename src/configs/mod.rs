// Application configuration modules.

pub mod app;
pub mod colors;
pub mod display;

pub mod hardware {
    //! Compatibility namespace for hardware configuration types.
    #[allow(unused_imports)]
    pub use super::app::{AppConfig, BleTpmsConfig, DisplayConfig, GpsConfig};
}

pub use app::AppConfig;
