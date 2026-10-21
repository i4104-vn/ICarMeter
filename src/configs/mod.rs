// Application configuration modules.

pub mod app;
pub mod colors;
pub mod config;
pub mod display;

pub use app::AppConfig;
#[allow(unused_imports)]
pub use config::{ensure_config_exists, get_config_path};
