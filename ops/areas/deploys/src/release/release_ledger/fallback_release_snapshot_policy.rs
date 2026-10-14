use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a fallback release snapshot code in its canonical FRS- form.
pub fn normalize_fallback_release_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FRS-") {
        return clean.clone();
    }
    format!("FRS-{}", clean.replace(" ", "-"))
}

pub fn fallback_release_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn fallback_release_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_release_snapshot() -> i64 {
    let policy = retry::default_policy("fallback-release-snapshot");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
