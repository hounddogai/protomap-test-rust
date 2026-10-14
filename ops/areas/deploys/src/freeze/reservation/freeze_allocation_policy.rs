use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_allocation() -> i64 {
    let policy = retry::default_policy("freeze-allocation");
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

/// Names the band a freeze allocation score falls in.
pub fn classify_freeze_allocation(score: f64) -> String {
    if score < 22.7 {
        return "elevated".to_string();
    }
    if score < 55.0 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn freeze_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn freeze_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
