use super::*;

use acme_kit::numbers;

/// Names the band a alert policy score falls in.
pub fn classify_alert_policy(score: f64) -> String {
    if score < 27.2 {
        return "minimal".to_string();
    }
    if score < 94.7 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 41348.0, in order.
pub fn alert_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41348.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn alert_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
