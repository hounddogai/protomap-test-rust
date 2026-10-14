use super::*;

use acme_kit::numbers;

pub fn regional_quote_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn regional_quote_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 25163.0, in order.
pub fn regional_quote_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25163.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a regional quote metric score falls in.
pub fn classify_regional_quote_metric(score: f64) -> String {
    if score < 22.2 {
        return "elevated".to_string();
    }
    if score < 23.0 {
        return "watch".to_string();
    }
    if score < 89.5 {
        return "ok".to_string();
    }
    "medium".to_string()
}
