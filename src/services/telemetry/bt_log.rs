// Bluetooth Logging Service: persistent logging and live log reader for dashboard display

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const BT_LOG_PATHS: &[&str] = &[
    "/var/log/icarmeter-bluetooth.log",
    "/dev/shm/icarmeter-bluetooth.log",
    "/tmp/icarmeter-bluetooth.log",
    "icarmeter-bluetooth.log",
];

fn get_writable_log_path() -> PathBuf {
    for path_str in BT_LOG_PATHS {
        let path = Path::new(path_str);
        // Try opening with append/create
        if let Ok(_) = OpenOptions::new().create(true).append(true).open(path) {
            return path.to_path_buf();
        }
    }
    PathBuf::from("/dev/shm/icarmeter-bluetooth.log")
}

fn get_readable_log_path() -> Option<PathBuf> {
    for path_str in BT_LOG_PATHS {
        let path = Path::new(path_str);
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > 0 {
                    return Some(path.to_path_buf());
                }
            }
        }
    }
    // If empty files exist, still return the first existing one
    for path_str in BT_LOG_PATHS {
        let path = Path::new(path_str);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn format_current_timestamp() -> String {
    #[cfg(unix)]
    unsafe {
        let mut now: libc::time_t = 0;
        libc::time(&mut now);
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&now, &mut tm);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        )
    }
    #[cfg(not(unix))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let s = secs % 60;
        let m = (secs / 60) % 60;
        let h = (secs / 3600) % 24;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}

/// Appends a structured log entry into the dedicated Bluetooth log file
pub fn log_bt(source: &str, message: &str) {
    let timestamp = format_current_timestamp();
    let line = format!("[{}] [{}] {}\n", timestamp, source, message);
    let path = get_writable_log_path();

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

/// Reads recent log entries (tail) up to `max_lines` from the Bluetooth log file
pub fn read_recent_bt_logs(max_lines: usize) -> Vec<String> {
    let Some(path) = get_readable_log_path() else {
        return vec![
            "[System] Bluetooth log initialized. Waiting for events...".to_string(),
            "[System] Listening on /var/log/icarmeter-bluetooth.log".to_string(),
            "[Hint] Press [B] to return to normal dashboard view".to_string(),
        ];
    };

    let Ok(file) = File::open(&path) else {
        return vec![
            format!("[Warning] Cannot open log file: {}", path.display()),
            "[Hint] Press [B] to return to normal dashboard view".to_string(),
        ];
    };

    let reader = BufReader::new(file);
    let mut all_lines = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            let trimmed = l.trim();
            if !trimmed.is_empty() {
                all_lines.push(trimmed.to_string());
            }
        }
    }

    if all_lines.is_empty() {
        return vec![
            "[System] Bluetooth log stream active (0 entries recorded yet).".to_string(),
            format!("[File] Active path: {}", path.display()),
            "[Hint] Press [B] to return to normal dashboard view".to_string(),
        ];
    }

    let start = all_lines.len().saturating_sub(max_lines);
    all_lines[start..].to_vec()
}

/// Cached log reader to avoid reading disk on every 60fps frame
pub struct BtLogReader {
    last_poll: Instant,
    cached_logs: Vec<String>,
    poll_interval: Duration,
}

impl BtLogReader {
    pub fn new() -> Self {
        Self {
            last_poll: Instant::now() - Duration::from_secs(10), // Force immediate initial read
            cached_logs: Vec::new(),
            poll_interval: Duration::from_millis(250),
        }
    }

    pub fn poll(&mut self, max_lines: usize) -> &[String] {
        if self.last_poll.elapsed() >= self.poll_interval || self.cached_logs.is_empty() {
            self.last_poll = Instant::now();
            self.cached_logs = read_recent_bt_logs(max_lines);
        }
        &self.cached_logs
    }
}
