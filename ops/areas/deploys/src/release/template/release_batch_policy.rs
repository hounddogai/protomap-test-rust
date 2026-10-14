use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_release_batch() -> i64 {
    let policy = retry::default_policy("release-batch");
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

/// Counts how many amounts fit in a budget of 65736.0, in order.
pub fn release_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 65736.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
