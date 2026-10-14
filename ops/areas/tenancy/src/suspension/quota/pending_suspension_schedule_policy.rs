use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_suspension_schedule() -> i64 {
    let policy = retry::default_policy("pending-suspension-schedule");
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

/// Names the band a pending suspension schedule score falls in.
pub fn classify_pending_suspension_schedule(score: f64) -> String {
    if score < 6.6 {
        return "watch".to_string();
    }
    if score < 83.1 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn pending_suspension_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn pending_suspension_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
