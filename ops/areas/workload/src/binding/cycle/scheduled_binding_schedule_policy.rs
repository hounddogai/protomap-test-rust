use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a scheduled binding schedule score falls in.
pub fn classify_scheduled_binding_schedule(score: f64) -> String {
    if score < 24.7 {
        return "low".to_string();
    }
    if score < 32.0 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_binding_schedule() -> i64 {
    let policy = retry::default_policy("scheduled-binding-schedule");
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

pub fn scheduled_binding_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
