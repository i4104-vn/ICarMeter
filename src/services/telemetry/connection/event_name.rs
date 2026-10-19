pub(super) fn clean_event_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let normalized = trimmed.to_ascii_lowercase();

    match normalized.as_str() {
        value if value.contains("connected") => "Connected",
        value if value.contains("telemetry") || value.contains("gps") => "Telemetry",
        value if value.contains("saved") || value.contains("set_config") => "Config Saved",
        value if value.contains("sync") || value.contains("get_config") => "Config Synced",
        value if value.contains("disconnect") => "Disconnected",
        value if value.contains("standby") || value.contains("idle") => "Standby",
        _ => {
            return trimmed
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(14)
                .collect();
        }
    }
    .to_string()
}
