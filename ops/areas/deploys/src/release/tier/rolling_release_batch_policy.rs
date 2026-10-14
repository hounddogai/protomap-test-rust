use super::*;

use acme_kit::numbers;

/// Names the band a rolling release batch score falls in.
pub fn classify_rolling_release_batch(score: f64) -> String {
    if score < 34.4 {
        return "minimal".to_string();
    }
    if score < 38.2 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn rolling_release_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn rolling_release_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 17285.0, in order.
pub fn rolling_release_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17285.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
