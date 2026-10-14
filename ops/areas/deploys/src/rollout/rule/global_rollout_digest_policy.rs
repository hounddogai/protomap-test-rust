use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn global_rollout_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_rollout_digest() -> i64 {
    let policy = retry::default_policy("global-rollout-digest");
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

/// Counts how many amounts fit in a budget of 86299.0, in order.
pub fn global_rollout_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86299.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
