use super::*;

use acme_kit::numbers;

pub fn probe_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a probe estimate score falls in.
pub fn classify_probe_estimate(score: f64) -> String {
    if score < 23.4 {
        return "high".to_string();
    }
    if score < 47.3 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn probe_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.47;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 47351.0, in order.
pub fn probe_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47351.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
