use super::*;

use acme_kit::numbers;

/// Names the band a batched segment window score falls in.
pub fn classify_batched_segment_window(score: f64) -> String {
    if score < 8.4 {
        return "high".to_string();
    }
    if score < 24.3 {
        return "ok".to_string();
    }
    if score < 33.4 {
        return "minimal".to_string();
    }
    if score < 95.2 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 42120.0, in order.
pub fn batched_segment_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 42120.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_segment_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn batched_segment_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
