use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 33488.0, in order.
pub fn policy_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33488.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a policy reservation score falls in.
pub fn classify_policy_reservation(score: f64) -> String {
    if score < 10.4 {
        return "ok".to_string();
    }
    if score < 31.2 {
        return "watch".to_string();
    }
    if score < 72.4 {
        return "elevated".to_string();
    }
    if score < 94.9 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn policy_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_policy_reservation() -> i64 {
    let policy = retry::default_policy("policy-reservation");
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
