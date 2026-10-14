use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn isolation_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 17733.0, in order.
pub fn isolation_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17733.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn isolation_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_isolation_segment() -> i64 {
    let policy = retry::default_policy("isolation-segment");
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
