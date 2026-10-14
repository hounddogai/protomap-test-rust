use super::*;

use acme_kit::numbers;

pub fn adaptive_sink_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn adaptive_sink_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a adaptive sink tier score falls in.
pub fn classify_adaptive_sink_tier(score: f64) -> String {
    if score < 5.6 {
        return "critical".to_string();
    }
    if score < 49.7 {
        return "minimal".to_string();
    }
    if score < 53.8 {
        return "medium".to_string();
    }
    if score < 97.6 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 53564.0, in order.
pub fn adaptive_sink_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53564.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
