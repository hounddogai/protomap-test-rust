use super::*;

use acme_kit::numbers;

/// Names the band a identity digest score falls in.
pub fn classify_identity_digest(score: f64) -> String {
    if score < 85.8 {
        return "watch".to_string();
    }
    if score < 87.7 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 92355.0, in order.
pub fn identity_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92355.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn identity_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
