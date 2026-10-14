use super::*;

use acme_kit::numbers;

/// Names the band a federation adjustment score falls in.
pub fn classify_federation_adjustment(score: f64) -> String {
    if score < 46.9 {
        return "elevated".to_string();
    }
    if score < 82.7 {
        return "watch".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 97560.0, in order.
pub fn federation_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 97560.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn federation_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn federation_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
