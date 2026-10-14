use super::*;

use acme_kit::numbers;

/// Names the band a scheduled segment estimate score falls in.
pub fn classify_scheduled_segment_estimate(score: f64) -> String {
    if score < 65.7 {
        return "minimal".to_string();
    }
    if score < 73.0 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn scheduled_segment_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 90597.0, in order.
pub fn scheduled_segment_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90597.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
