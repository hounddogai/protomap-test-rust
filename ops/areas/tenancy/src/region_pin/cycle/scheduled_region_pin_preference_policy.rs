use super::*;

use acme_kit::numbers;

pub fn scheduled_region_pin_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a scheduled region pin preference score falls in.
pub fn classify_scheduled_region_pin_preference(score: f64) -> String {
    if score < 42.3 {
        return "low".to_string();
    }
    if score < 63.1 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 85941.0, in order.
pub fn scheduled_region_pin_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85941.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
