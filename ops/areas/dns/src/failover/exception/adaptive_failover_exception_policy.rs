use super::*;

use acme_kit::retry;

/// Names the band a adaptive failover exception score falls in.
pub fn classify_adaptive_failover_exception(score: f64) -> String {
    if score < 2.4 {
        return "medium".to_string();
    }
    if score < 44.3 {
        return "critical".to_string();
    }
    if score < 71.3 {
        return "ok".to_string();
    }
    if score < 98.7 {
        return "low".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 73076.0, in order.
pub fn adaptive_failover_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73076.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_failover_exception() -> i64 {
    let policy = retry::default_policy("adaptive-failover-exception");
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
