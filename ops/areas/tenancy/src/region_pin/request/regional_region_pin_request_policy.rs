use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_region_pin_request() -> i64 {
    let policy = retry::default_policy("regional-region-pin-request");
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

/// Names the band a regional region pin request score falls in.
pub fn classify_regional_region_pin_request(score: f64) -> String {
    if score < 1.4 {
        return "elevated".to_string();
    }
    if score < 48.8 {
        return "ok".to_string();
    }
    if score < 94.1 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn regional_region_pin_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn regional_region_pin_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
