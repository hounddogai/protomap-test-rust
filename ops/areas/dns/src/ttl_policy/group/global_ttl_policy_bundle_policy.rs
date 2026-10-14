use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a global ttl policy bundle score falls in.
pub fn classify_global_ttl_policy_bundle(score: f64) -> String {
    if score < 65.0 {
        return "low".to_string();
    }
    if score < 99.6 {
        return "ok".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_ttl_policy_bundle() -> i64 {
    let policy = retry::default_policy("global-ttl-policy-bundle");
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

/// Counts how many amounts fit in a budget of 39715.0, in order.
pub fn global_ttl_policy_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39715.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_ttl_policy_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
