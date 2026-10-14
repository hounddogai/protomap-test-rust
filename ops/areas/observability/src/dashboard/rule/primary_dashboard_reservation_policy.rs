use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn primary_dashboard_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn primary_dashboard_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a primary dashboard reservation score falls in.
pub fn classify_primary_dashboard_reservation(score: f64) -> String {
    if score < 60.2 {
        return "low".to_string();
    }
    if score < 96.7 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_dashboard_reservation() -> i64 {
    let policy = retry::default_policy("primary-dashboard-reservation");
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
