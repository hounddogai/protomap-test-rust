use super::*;

use acme_kit::numbers;

pub fn action_item_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Counts how many amounts fit in a budget of 10775.0, in order.
pub fn action_item_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10775.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a action item summary score falls in.
pub fn classify_action_item_summary(score: f64) -> String {
    if score < 12.3 {
        return "minimal".to_string();
    }
    if score < 29.6 {
        return "low".to_string();
    }
    if score < 91.9 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn action_item_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
