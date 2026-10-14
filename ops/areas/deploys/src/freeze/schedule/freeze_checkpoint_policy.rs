use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_checkpoint() -> i64 {
    let policy = retry::default_policy("freeze-checkpoint");
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

pub fn freeze_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a freeze checkpoint score falls in.
pub fn classify_freeze_checkpoint(score: f64) -> String {
    if score < 12.9 {
        return "high".to_string();
    }
    if score < 39.1 {
        return "ok".to_string();
    }
    if score < 89.7 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn freeze_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
