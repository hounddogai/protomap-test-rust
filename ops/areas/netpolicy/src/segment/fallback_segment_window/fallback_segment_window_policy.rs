use super::*;

use acme_kit::numbers;

pub fn fallback_segment_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 46687.0, in order.
pub fn fallback_segment_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46687.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a fallback segment window score falls in.
pub fn classify_fallback_segment_window(score: f64) -> String {
    if score < 79.4 {
        return "ok".to_string();
    }
    if score < 89.1 {
        return "high".to_string();
    }
    "critical".to_string()
}
