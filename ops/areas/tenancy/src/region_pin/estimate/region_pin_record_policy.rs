use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn region_pin_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_region_pin_record() -> i64 {
    let policy = retry::default_policy("region-pin-record");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn region_pin_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a region pin record code in its canonical RPR- form.
pub fn normalize_region_pin_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPR-") {
        return clean.clone();
    }
    format!("RPR-{}", clean.replace(" ", "-"))
}
