use super::*;

use acme_kit::retry;

/// Names the band a binding exception score falls in.
pub fn classify_binding_exception(score: f64) -> String {
    if score < 1.3 {
        return "medium".to_string();
    }
    if score < 64.2 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 15399.0, in order.
pub fn binding_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15399.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_binding_exception() -> i64 {
    let policy = retry::default_policy("binding-exception");
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

pub fn binding_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
