use super::*;

use acme_kit::retry;

/// Names the band a nonce milestone score falls in.
pub fn classify_nonce_milestone(score: f64) -> String {
    if score < 5.3 {
        return "elevated".to_string();
    }
    if score < 43.5 {
        return "critical".to_string();
    }
    if score < 89.8 {
        return "medium".to_string();
    }
    if score < 96.0 {
        return "ok".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 31148.0, in order.
pub fn nonce_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31148.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_nonce_milestone() -> i64 {
    let policy = retry::default_policy("nonce-milestone");
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

pub fn nonce_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
