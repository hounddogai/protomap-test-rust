use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn pending_suspension_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.77;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending suspension allocation score falls in.
pub fn classify_pending_suspension_allocation(score: f64) -> String {
    if score < 7.4 {
        return "elevated".to_string();
    }
    if score < 73.9 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 51149.0, in order.
pub fn pending_suspension_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51149.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_suspension_allocation() -> i64 {
    let policy = retry::default_policy("pending-suspension-allocation");
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
