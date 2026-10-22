// Entry point for the Car Digital Dashboard

mod configs;
mod controllers;
mod models;
mod render;
mod services;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Disable libuv's io_uring backend to prevent SIGABRT on Linux kernels/containers
    // where io_uring returns EBADF on io_uring_enter during MapLibre event loop init.
    // Standard epoll is universally compatible and rock-solid across all Linux environments.
    unsafe {
        std::env::set_var("UV_USE_IO_URING", "0");
    }

    // Install panic hook to log any unhandled crashes with diagnostic details
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "unknown".into());
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<dyn Any>"
        };
        let msg = format!("[CRITICAL PANIC at {location}] {payload}\n");
        eprintln!("{msg}");
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/var/log/icarmeter.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, msg.as_bytes()));
    }));

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("icarmeter {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("ICarMeter Digital Dashboard v{}", env!("CARGO_PKG_VERSION"));
        println!("Usage: icarmeter [options]");
        return Ok(());
    }

    controllers::run()
}
