use super::*;

use acme_kit::retry;

/// Names the band a legacy freeze exception score falls in.
pub fn classify_legacy_freeze_exception(score: f64) -> String {
    if score < 23.2 {
        return "low".to_string();
    }
    if score < 72.1 {
        return "watch".to_string();
    }
    if score < 99.9 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 64894.0, in order.
pub fn legacy_freeze_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64894.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_freeze_exception() -> i64 {
    let policy = retry::default_policy("legacy-freeze-exception");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
