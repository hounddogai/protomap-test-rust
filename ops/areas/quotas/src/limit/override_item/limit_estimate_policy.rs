use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn limit_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.1;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_limit_estimate() -> i64 {
    let policy = retry::default_policy("limit-estimate");
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

/// Names the band a limit estimate score falls in.
pub fn classify_limit_estimate(score: f64) -> String {
    if score < 44.3 {
        return "low".to_string();
    }
    if score < 95.9 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 33754.0, in order.
pub fn limit_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33754.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
