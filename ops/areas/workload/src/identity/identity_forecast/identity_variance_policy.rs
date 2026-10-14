use super::*;

use acme_kit::numbers;

pub fn identity_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn identity_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a identity variance score falls in.
pub fn classify_identity_variance(score: f64) -> String {
    if score < 38.0 {
        return "ok".to_string();
    }
    if score < 76.0 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 3965.0, in order.
pub fn identity_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3965.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
