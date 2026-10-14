use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn timeline_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 35129.0, in order.
pub fn timeline_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35129.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn timeline_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_timeline_milestone() -> i64 {
    let policy = retry::default_policy("timeline-milestone");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
