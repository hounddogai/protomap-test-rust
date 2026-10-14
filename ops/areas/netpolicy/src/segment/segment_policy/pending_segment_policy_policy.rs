use super::*;

use acme_kit::numbers;

pub fn pending_segment_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn pending_segment_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a pending segment policy score falls in.
pub fn classify_pending_segment_policy(score: f64) -> String {
    if score < 64.6 {
        return "low".to_string();
    }
    if score < 66.7 {
        return "minimal".to_string();
    }
    if score < 87.5 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 83816.0, in order.
pub fn pending_segment_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83816.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
