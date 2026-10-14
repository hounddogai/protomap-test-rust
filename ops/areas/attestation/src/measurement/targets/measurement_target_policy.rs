use super::*;

use acme_kit::numbers;

/// Names the band a measurement target score falls in.
pub fn classify_measurement_target(score: f64) -> String {
    if score < 75.6 {
        return "high".to_string();
    }
    if score < 92.5 {
        return "watch".to_string();
    }
    if score < 93.7 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 47032.0, in order.
pub fn measurement_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47032.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn measurement_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
