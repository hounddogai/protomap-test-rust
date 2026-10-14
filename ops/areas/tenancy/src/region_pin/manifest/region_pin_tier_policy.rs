use super::*;

use acme_kit::numbers;

pub fn region_pin_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.73;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a region pin tier score falls in.
pub fn classify_region_pin_tier(score: f64) -> String {
    if score < 3.8 {
        return "low".to_string();
    }
    if score < 30.1 {
        return "ok".to_string();
    }
    if score < 40.1 {
        return "elevated".to_string();
    }
    if score < 98.1 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 69015.0, in order.
pub fn region_pin_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69015.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn region_pin_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
