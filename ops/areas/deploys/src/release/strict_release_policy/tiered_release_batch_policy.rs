use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 88267.0, in order.
pub fn tiered_release_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88267.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_release_batch() -> i64 {
    let policy = retry::default_policy("tiered-release-batch");
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

/// Names the band a tiered release batch score falls in.
pub fn classify_tiered_release_batch(score: f64) -> String {
    if score < 39.1 {
        return "medium".to_string();
    }
    if score < 88.1 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
