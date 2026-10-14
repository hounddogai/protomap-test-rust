use super::*;

use acme_kit::numbers;

/// Names the band a freeze batch score falls in.
pub fn classify_freeze_batch(score: f64) -> String {
    if score < 7.8 {
        return "critical".to_string();
    }
    if score < 64.9 {
        return "watch".to_string();
    }
    if score < 78.5 {
        return "high".to_string();
    }
    if score < 79.8 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn freeze_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 38703.0, in order.
pub fn freeze_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38703.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn freeze_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
