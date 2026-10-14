use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_segment_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.77;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a fallback segment entry score falls in.
pub fn classify_fallback_segment_entry(score: f64) -> String {
    if score < 36.3 {
        return "elevated".to_string();
    }
    if score < 57.0 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_segment_entry() -> i64 {
    let policy = retry::default_policy("fallback-segment-entry");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 50652.0, in order.
pub fn fallback_segment_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50652.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
