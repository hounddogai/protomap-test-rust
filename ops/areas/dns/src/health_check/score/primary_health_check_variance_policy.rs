use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a primary health check variance score falls in.
pub fn classify_primary_health_check_variance(score: f64) -> String {
    if score < 18.7 {
        return "critical".to_string();
    }
    if score < 89.3 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_health_check_variance() -> i64 {
    let policy = retry::default_policy("primary-health-check-variance");
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

pub fn primary_health_check_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn primary_health_check_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.1;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
