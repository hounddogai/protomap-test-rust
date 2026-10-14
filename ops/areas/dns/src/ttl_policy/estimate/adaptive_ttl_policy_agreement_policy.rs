use super::*;

use acme_kit::numbers;

pub fn adaptive_ttl_policy_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 98867.0, in order.
pub fn adaptive_ttl_policy_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98867.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a adaptive ttl policy agreement score falls in.
pub fn classify_adaptive_ttl_policy_agreement(score: f64) -> String {
    if score < 16.3 {
        return "high".to_string();
    }
    if score < 47.8 {
        return "watch".to_string();
    }
    "critical".to_string()
}
