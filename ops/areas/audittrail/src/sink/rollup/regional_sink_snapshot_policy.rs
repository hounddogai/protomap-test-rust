use super::*;

use acme_kit::retry;

/// Puts a regional sink snapshot code in its canonical RSS- form.
pub fn normalize_regional_sink_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RSS-") {
        return clean.clone();
    }
    format!("RSS-{}", clean.replace(" ", "-"))
}

pub fn regional_sink_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_sink_snapshot() -> i64 {
    let policy = retry::default_policy("regional-sink-snapshot");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
