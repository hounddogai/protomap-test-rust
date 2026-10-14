use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 53267.0, in order.
pub fn legacy_alert_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53267.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a legacy alert slot score falls in.
pub fn classify_legacy_alert_slot(score: f64) -> String {
    if score < 48.2 {
        return "high".to_string();
    }
    if score < 68.7 {
        return "critical".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_alert_slot() -> i64 {
    let policy = retry::default_policy("legacy-alert-slot");
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
