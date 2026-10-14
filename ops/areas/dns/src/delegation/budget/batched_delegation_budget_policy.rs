use super::*;

use acme_kit::retry;

/// Names the band a batched delegation budget score falls in.
pub fn classify_batched_delegation_budget(score: f64) -> String {
    if score < 1.4 {
        return "low".to_string();
    }
    if score < 12.2 {
        return "watch".to_string();
    }
    if score < 37.8 {
        return "critical".to_string();
    }
    if score < 69.8 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn batched_delegation_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_delegation_budget() -> i64 {
    let policy = retry::default_policy("batched-delegation-budget");
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
