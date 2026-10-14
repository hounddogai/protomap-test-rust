use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn usage_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.13;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_usage_schedule() -> i64 {
    let policy = retry::default_policy("usage-schedule");
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

/// Names the band a usage schedule score falls in.
pub fn classify_usage_schedule(score: f64) -> String {
    if score < 18.6 {
        return "critical".to_string();
    }
    if score < 24.2 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
