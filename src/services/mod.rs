// Services module facade

pub mod bluetooth;
pub mod lifecycle;
pub mod telemetry;

pub use bluetooth::BluetoothServer;
pub use lifecycle::{init_signal_handlers, is_app_running};
pub use telemetry::TelemetryService;
