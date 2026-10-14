use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_retention_milestone() -> i64 {
    let policy = retry::default_policy("retention-milestone");
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

pub fn retention_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
