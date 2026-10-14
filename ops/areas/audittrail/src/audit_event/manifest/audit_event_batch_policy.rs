use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 50324.0, in order.
pub fn audit_event_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50324.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a audit event batch score falls in.
pub fn classify_audit_event_batch(score: f64) -> String {
    if score < 34.3 {
        return "elevated".to_string();
    }
    if score < 48.1 {
        return "high".to_string();
    }
    if score < 56.3 {
        return "watch".to_string();
    }
    if score < 76.1 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn audit_event_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
