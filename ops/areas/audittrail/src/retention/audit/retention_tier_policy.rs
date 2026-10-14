use super::*;

use acme_kit::numbers;

/// Names the band a retention tier score falls in.
pub fn classify_retention_tier(score: f64) -> String {
    if score < 23.0 {
        return "watch".to_string();
    }
    if score < 51.2 {
        return "high".to_string();
    }
    if score < 70.0 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn retention_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 68262.0, in order.
pub fn retention_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68262.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
