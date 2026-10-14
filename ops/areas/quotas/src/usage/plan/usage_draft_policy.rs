use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn usage_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn usage_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_usage_draft() -> i64 {
    let policy = retry::default_policy("usage-draft");
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

/// Names the band a usage draft score falls in.
pub fn classify_usage_draft(score: f64) -> String {
    if score < 25.7 {
        return "medium".to_string();
    }
    if score < 66.9 {
        return "ok".to_string();
    }
    "watch".to_string()
}
