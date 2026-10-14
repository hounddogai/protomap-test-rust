use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a overage bundle score falls in.
pub fn classify_overage_bundle(score: f64) -> String {
    if score < 4.7 {
        return "critical".to_string();
    }
    if score < 33.0 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn overage_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_overage_bundle() -> i64 {
    let policy = retry::default_policy("overage-bundle");
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

/// Counts how many amounts fit in a budget of 93318.0, in order.
pub fn overage_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93318.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
