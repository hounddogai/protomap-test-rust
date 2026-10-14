use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 26513.0, in order.
pub fn strict_credential_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 26513.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a strict credential segment score falls in.
pub fn classify_strict_credential_segment(score: f64) -> String {
    if score < 44.9 {
        return "medium".to_string();
    }
    if score < 64.0 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_credential_segment() -> i64 {
    let policy = retry::default_policy("strict-credential-segment");
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
