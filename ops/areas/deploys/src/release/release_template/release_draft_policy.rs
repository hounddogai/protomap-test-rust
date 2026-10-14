use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_release_draft() -> i64 {
    let policy = retry::default_policy("release-draft");
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

/// Counts how many amounts fit in a budget of 49925.0, in order.
pub fn release_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49925.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a release draft score falls in.
pub fn classify_release_draft(score: f64) -> String {
    if score < 3.2 {
        return "minimal".to_string();
    }
    if score < 3.4 {
        return "critical".to_string();
    }
    if score < 39.1 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn release_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
