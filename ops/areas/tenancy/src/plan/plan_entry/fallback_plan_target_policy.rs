use super::*;

use acme_kit::numbers;

pub fn fallback_plan_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.08;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_plan_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Counts how many amounts fit in a budget of 38164.0, in order.
pub fn fallback_plan_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38164.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a fallback plan target score falls in.
pub fn classify_fallback_plan_target(score: f64) -> String {
    if score < 45.6 {
        return "watch".to_string();
    }
    if score < 79.4 {
        return "low".to_string();
    }
    "high".to_string()
}
