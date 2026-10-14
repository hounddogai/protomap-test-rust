use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 70196.0, in order.
pub fn ttl_policy_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70196.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_rollup() -> i64 {
    let policy = retry::default_policy("ttl-policy-rollup");
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

/// Names the band a ttl policy rollup score falls in.
pub fn classify_ttl_policy_rollup(score: f64) -> String {
    if score < 18.7 {
        return "critical".to_string();
    }
    if score < 70.5 {
        return "elevated".to_string();
    }
    "low".to_string()
}
