use super::*;

use acme_kit::numbers;

/// Names the band a federation variance score falls in.
pub fn classify_federation_variance(score: f64) -> String {
    if score < 12.1 {
        return "medium".to_string();
    }
    if score < 25.4 {
        return "watch".to_string();
    }
    if score < 27.0 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn federation_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn federation_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 33445.0, in order.
pub fn federation_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33445.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
