use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 16541.0, in order.
pub fn binding_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16541.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_binding_rollup() -> i64 {
    let policy = retry::default_policy("binding-rollup");
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

/// Names the band a binding rollup score falls in.
pub fn classify_binding_rollup(score: f64) -> String {
    if score < 50.0 {
        return "medium".to_string();
    }
    if score < 69.7 {
        return "critical".to_string();
    }
    "high".to_string()
}
