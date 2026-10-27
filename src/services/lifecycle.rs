//! Process lifetime flag shared by the telemetry worker and UI runtime.

#[allow(unused_imports)]
pub use crate::services::input::is_app_running;

pub fn init_signal_handlers() {
    // Initialized cleanly in TtyManager
}
