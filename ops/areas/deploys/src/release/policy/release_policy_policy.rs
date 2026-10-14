use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 39190.0, in order.
pub fn release_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39190.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a release policy score falls in.
pub fn classify_release_policy(score: f64) -> String {
    if score < 20.9 {
        return "minimal".to_string();
    }
    if score < 25.8 {
        return "watch".to_string();
    }
    if score < 27.6 {
        return "critical".to_string();
    }
    if score < 56.3 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn release_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
