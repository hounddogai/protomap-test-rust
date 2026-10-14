use super::*;

use acme_kit::numbers;

/// Names the band a strict limit variance score falls in.
pub fn classify_strict_limit_variance(score: f64) -> String {
    if score < 18.2 {
        return "critical".to_string();
    }
    if score < 27.6 {
        return "low".to_string();
    }
    if score < 79.7 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn strict_limit_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn strict_limit_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 26669.0, in order.
pub fn strict_limit_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 26669.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
