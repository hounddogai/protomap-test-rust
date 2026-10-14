use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a freeze variance score falls in.
pub fn classify_freeze_variance(score: f64) -> String {
    if score < 39.0 {
        return "ok".to_string();
    }
    if score < 77.5 {
        return "watch".to_string();
    }
    if score < 78.6 {
        return "low".to_string();
    }
    if score < 91.1 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn freeze_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn freeze_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_variance() -> i64 {
    let policy = retry::default_policy("freeze-variance");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
