use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_target() -> i64 {
    let policy = retry::default_policy("ttl-policy-target");
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

pub fn ttl_policy_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.18;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a ttl policy target score falls in.
pub fn classify_ttl_policy_target(score: f64) -> String {
    if score < 67.3 {
        return "minimal".to_string();
    }
    if score < 74.2 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 86899.0, in order.
pub fn ttl_policy_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86899.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
