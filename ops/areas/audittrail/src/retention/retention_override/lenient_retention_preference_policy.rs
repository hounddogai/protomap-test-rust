use super::*;

use acme_kit::retry;

/// Names the band a lenient retention preference score falls in.
pub fn classify_lenient_retention_preference(score: f64) -> String {
    if score < 8.3 {
        return "high".to_string();
    }
    if score < 23.7 {
        return "ok".to_string();
    }
    if score < 34.1 {
        return "elevated".to_string();
    }
    if score < 60.2 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_retention_preference() -> i64 {
    let policy = retry::default_policy("lenient-retention-preference");
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

/// Counts how many amounts fit in a budget of 6242.0, in order.
pub fn lenient_retention_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6242.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
