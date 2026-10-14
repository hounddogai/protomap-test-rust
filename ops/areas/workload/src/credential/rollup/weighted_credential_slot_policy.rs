use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 12832.0, in order.
pub fn weighted_credential_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12832.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a weighted credential slot score falls in.
pub fn classify_weighted_credential_slot(score: f64) -> String {
    if score < 46.4 {
        return "medium".to_string();
    }
    if score < 59.7 {
        return "low".to_string();
    }
    if score < 71.0 {
        return "high".to_string();
    }
    if score < 81.3 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn weighted_credential_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.81;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
