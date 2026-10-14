use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_release_review() -> i64 {
    let policy = retry::default_policy("strict-release-review");
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

/// Counts how many amounts fit in a budget of 2640.0, in order.
pub fn strict_release_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2640.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
