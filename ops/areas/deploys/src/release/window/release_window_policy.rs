use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_release_window() -> i64 {
    let policy = retry::default_policy("release-window");
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

/// Names the band a release window score falls in.
pub fn classify_release_window(score: f64) -> String {
    if score < 27.1 {
        return "ok".to_string();
    }
    if score < 60.9 {
        return "elevated".to_string();
    }
    if score < 93.4 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 96095.0, in order.
pub fn release_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 96095.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
