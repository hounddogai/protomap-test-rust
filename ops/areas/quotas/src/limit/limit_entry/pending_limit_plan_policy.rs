use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_limit_plan() -> i64 {
    let policy = retry::default_policy("pending-limit-plan");
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

pub fn pending_limit_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a pending limit plan score falls in.
pub fn classify_pending_limit_plan(score: f64) -> String {
    if score < 56.6 {
        return "watch".to_string();
    }
    if score < 65.3 {
        return "critical".to_string();
    }
    if score < 95.5 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn pending_limit_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.93;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
