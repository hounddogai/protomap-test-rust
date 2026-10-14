use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a pending region pin segment code in its canonical PRPS- form.
pub fn normalize_pending_region_pin_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRPS-") {
        return clean.clone();
    }
    format!("PRPS-{}", clean.replace(" ", "-"))
}

/// Names the band a pending region pin segment score falls in.
pub fn classify_pending_region_pin_segment(score: f64) -> String {
    if score < 6.3 {
        return "elevated".to_string();
    }
    if score < 87.5 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_region_pin_segment() -> i64 {
    let policy = retry::default_policy("pending-region-pin-segment");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn pending_region_pin_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
