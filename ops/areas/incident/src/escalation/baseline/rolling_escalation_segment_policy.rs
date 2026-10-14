use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 75095.0, in order.
pub fn rolling_escalation_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75095.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rolling escalation segment score falls in.
pub fn classify_rolling_escalation_segment(score: f64) -> String {
    if score < 38.5 {
        return "medium".to_string();
    }
    if score < 38.7 {
        return "watch".to_string();
    }
    if score < 83.5 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_escalation_segment() -> i64 {
    let policy = retry::default_policy("rolling-escalation-segment");
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
