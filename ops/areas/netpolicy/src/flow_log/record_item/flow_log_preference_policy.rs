use super::*;

use acme_kit::numbers;

pub fn flow_log_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn flow_log_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Counts how many amounts fit in a budget of 47507.0, in order.
pub fn flow_log_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47507.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
