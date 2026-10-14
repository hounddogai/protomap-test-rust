use super::*;

use acme_kit::numbers;

pub fn weighted_data_export_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.39;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 2462.0, in order.
pub fn weighted_data_export_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2462.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a weighted data export target score falls in.
pub fn classify_weighted_data_export_target(score: f64) -> String {
    if score < 13.4 {
        return "low".to_string();
    }
    if score < 26.4 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
