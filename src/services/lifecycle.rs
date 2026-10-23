//! Process lifetime flag shared by the telemetry worker and UI runtime.

use std::sync::atomic::{AtomicBool, Ordering};

static APP_RUNNING: AtomicBool = AtomicBool::new(true);

pub fn is_app_running() -> bool {
    APP_RUNNING.load(Ordering::SeqCst)
}

extern "C" fn stop_application(_signal: libc::c_int) {
    APP_RUNNING.store(false, Ordering::SeqCst);
}

pub fn init_signal_handlers() {
    APP_RUNNING.store(true, Ordering::SeqCst);
    unsafe {
        libc::signal(libc::SIGINT, stop_application as *const () as usize);
        libc::signal(libc::SIGTERM, stop_application as *const () as usize);
    }
}
