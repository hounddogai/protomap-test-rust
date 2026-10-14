use super::*;

use acme_kit::numbers;

pub fn manual_budget_burn_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn manual_budget_burn_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a manual budget burn entry score falls in.
pub fn classify_manual_budget_burn_entry(score: f64) -> String {
    if score < 8.5 {
        return "medium".to_string();
    }
    if score < 62.9 {
        return "watch".to_string();
    }
    if score < 89.7 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 93846.0, in order.
pub fn manual_budget_burn_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93846.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
