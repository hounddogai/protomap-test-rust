use super::*;

use acme_kit::numbers;

/// Names the band a batched token notice score falls in.
pub fn classify_batched_token_notice(score: f64) -> String {
    if score < 17.5 {
        return "elevated".to_string();
    }
    if score < 44.6 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn batched_token_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 88937.0, in order.
pub fn batched_token_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88937.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
