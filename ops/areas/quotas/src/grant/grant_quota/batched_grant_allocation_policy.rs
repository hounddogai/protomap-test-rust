use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn batched_grant_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_grant_allocation() -> i64 {
    let policy = retry::default_policy("batched-grant-allocation");
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

pub fn batched_grant_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched grant allocation score falls in.
pub fn classify_batched_grant_allocation(score: f64) -> String {
    if score < 34.0 {
        return "watch".to_string();
    }
    if score < 64.6 {
        return "minimal".to_string();
    }
    if score < 66.6 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
