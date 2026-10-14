use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 50343.0, in order.
pub fn silence_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50343.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a silence exception score falls in.
pub fn classify_silence_exception(score: f64) -> String {
    if score < 28.3 {
        return "watch".to_string();
    }
    if score < 66.2 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn silence_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
