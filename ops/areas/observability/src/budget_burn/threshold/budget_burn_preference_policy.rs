use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 55760.0, in order.
pub fn budget_burn_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55760.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn budget_burn_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn budget_burn_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a budget burn preference score falls in.
pub fn classify_budget_burn_preference(score: f64) -> String {
    if score < 22.7 {
        return "elevated".to_string();
    }
    if score < 63.0 {
        return "watch".to_string();
    }
    if score < 71.1 {
        return "high".to_string();
    }
    if score < 83.3 {
        return "medium".to_string();
    }
    "critical".to_string()
}
