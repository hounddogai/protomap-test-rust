use super::*;

use acme_kit::numbers;

/// Names the band a batched binding threshold score falls in.
pub fn classify_batched_binding_threshold(score: f64) -> String {
    if score < 23.3 {
        return "low".to_string();
    }
    if score < 34.1 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn batched_binding_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 68395.0, in order.
pub fn batched_binding_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68395.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
