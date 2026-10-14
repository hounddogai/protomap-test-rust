use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 76946.0, in order.
pub fn scheduled_quote_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76946.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a scheduled quote group score falls in.
pub fn classify_scheduled_quote_group(score: f64) -> String {
    if score < 59.3 {
        return "low".to_string();
    }
    if score < 93.2 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn scheduled_quote_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
