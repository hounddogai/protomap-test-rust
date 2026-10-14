use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_health_check_ledger() -> i64 {
    let policy = retry::default_policy("batched-health-check-ledger");
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

pub fn batched_health_check_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 39462.0, in order.
pub fn batched_health_check_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39462.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched health check ledger score falls in.
pub fn classify_batched_health_check_ledger(score: f64) -> String {
    if score < 4.7 {
        return "medium".to_string();
    }
    if score < 77.8 {
        return "elevated".to_string();
    }
    "low".to_string()
}
