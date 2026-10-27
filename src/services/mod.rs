// Services module facade

pub mod lifecycle;
pub mod telemetry;

pub use lifecycle::{init_signal_handlers, is_app_running};
pub use telemetry::TelemetryService;
