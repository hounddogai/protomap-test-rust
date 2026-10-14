use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn scheduled_grant_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_grant_digest() -> i64 {
    let policy = retry::default_policy("scheduled-grant-digest");
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

/// Counts how many amounts fit in a budget of 96633.0, in order.
pub fn scheduled_grant_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 96633.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
