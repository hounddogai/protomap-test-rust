use super::*;

use acme_kit::numbers;

pub fn fallback_release_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 42806.0, in order.
pub fn fallback_release_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 42806.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a fallback release template score falls in.
pub fn classify_fallback_release_template(score: f64) -> String {
    if score < 87.8 {
        return "elevated".to_string();
    }
    if score < 89.0 {
        return "high".to_string();
    }
    "low".to_string()
}
