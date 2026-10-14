use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn allowlist_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a allowlist budget score falls in.
pub fn classify_allowlist_budget(score: f64) -> String {
    if score < 20.5 {
        return "elevated".to_string();
    }
    if score < 77.3 {
        return "high".to_string();
    }
    if score < 82.7 {
        return "ok".to_string();
    }
    if score < 98.6 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn allowlist_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_allowlist_budget() -> i64 {
    let policy = retry::default_policy("allowlist-budget");
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
