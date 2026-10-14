use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 54834.0, in order.
pub fn federation_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54834.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn federation_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.5;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a federation schedule score falls in.
pub fn classify_federation_schedule(score: f64) -> String {
    if score < 2.7 {
        return "high".to_string();
    }
    if score < 35.0 {
        return "critical".to_string();
    }
    if score < 47.5 {
        return "medium".to_string();
    }
    if score < 52.6 {
        return "low".to_string();
    }
    "watch".to_string()
}
