use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_usage_group() -> i64 {
    let policy = retry::default_policy("capped-usage-group");
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

pub fn capped_usage_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a capped usage group score falls in.
pub fn classify_capped_usage_group(score: f64) -> String {
    if score < 43.8 {
        return "elevated".to_string();
    }
    if score < 78.3 {
        return "watch".to_string();
    }
    if score < 95.4 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn capped_usage_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
