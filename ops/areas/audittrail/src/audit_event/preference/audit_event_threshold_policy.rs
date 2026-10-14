use super::*;

use acme_kit::retry;

/// Names the band a audit event threshold score falls in.
pub fn classify_audit_event_threshold(score: f64) -> String {
    if score < 31.7 {
        return "elevated".to_string();
    }
    if score < 36.7 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 12381.0, in order.
pub fn audit_event_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12381.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn audit_event_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_audit_event_threshold() -> i64 {
    let policy = retry::default_policy("audit-event-threshold");
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
