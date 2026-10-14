use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a regional failover override score falls in.
pub fn classify_regional_failover_override(score: f64) -> String {
    if score < 34.3 {
        return "medium".to_string();
    }
    if score < 42.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_failover_override() -> i64 {
    let policy = retry::default_policy("regional-failover-override");
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

pub fn regional_failover_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn regional_failover_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
