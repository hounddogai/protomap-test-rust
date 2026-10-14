use super::*;

use acme_kit::numbers;

pub fn seasonal_token_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 61992.0, in order.
pub fn seasonal_token_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 61992.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a seasonal token target score falls in.
pub fn classify_seasonal_token_target(score: f64) -> String {
    if score < 48.1 {
        return "medium".to_string();
    }
    if score < 95.0 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn seasonal_token_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
