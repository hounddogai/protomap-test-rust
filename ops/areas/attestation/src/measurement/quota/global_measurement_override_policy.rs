use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 44794.0, in order.
pub fn global_measurement_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44794.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_measurement_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a global measurement override score falls in.
pub fn classify_global_measurement_override(score: f64) -> String {
    if score < 15.1 {
        return "medium".to_string();
    }
    if score < 25.4 {
        return "high".to_string();
    }
    if score < 70.9 {
        return "ok".to_string();
    }
    if score < 78.2 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_measurement_override() -> i64 {
    let policy = retry::default_policy("global-measurement-override");
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
