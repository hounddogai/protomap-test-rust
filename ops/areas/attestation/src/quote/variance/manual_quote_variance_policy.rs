use super::*;

use acme_kit::numbers;

/// Names the band a manual quote variance score falls in.
pub fn classify_manual_quote_variance(score: f64) -> String {
    if score < 46.1 {
        return "critical".to_string();
    }
    if score < 70.1 {
        return "high".to_string();
    }
    if score < 75.6 {
        return "low".to_string();
    }
    if score < 82.9 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn manual_quote_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 14387.0, in order.
pub fn manual_quote_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14387.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
