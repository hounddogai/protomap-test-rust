use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 17360.0, in order.
pub fn rule_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17360.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_window() -> i64 {
    let policy = retry::default_policy("rule-window");
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

/// Names the band a rule window score falls in.
pub fn classify_rule_window(score: f64) -> String {
    if score < 11.9 {
        return "ok".to_string();
    }
    if score < 63.7 {
        return "low".to_string();
    }
    if score < 69.9 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
