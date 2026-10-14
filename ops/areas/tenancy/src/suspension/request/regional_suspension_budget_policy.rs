use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 59722.0, in order.
pub fn regional_suspension_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 59722.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a regional suspension budget score falls in.
pub fn classify_regional_suspension_budget(score: f64) -> String {
    if score < 34.8 {
        return "ok".to_string();
    }
    if score < 50.7 {
        return "elevated".to_string();
    }
    if score < 89.5 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn regional_suspension_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
