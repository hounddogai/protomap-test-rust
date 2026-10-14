use super::*;

use acme_kit::retry;

/// Names the band a batched policy window score falls in.
pub fn classify_batched_policy_window(score: f64) -> String {
    if score < 1.5 {
        return "low".to_string();
    }
    if score < 7.1 {
        return "critical".to_string();
    }
    if score < 66.9 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_policy_window() -> i64 {
    let policy = retry::default_policy("batched-policy-window");
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

/// Counts how many amounts fit in a budget of 33847.0, in order.
pub fn batched_policy_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33847.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
