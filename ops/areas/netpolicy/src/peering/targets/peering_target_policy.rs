use super::*;

use acme_kit::numbers;

pub fn peering_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a peering target score falls in.
pub fn classify_peering_target(score: f64) -> String {
    if score < 30.3 {
        return "medium".to_string();
    }
    if score < 85.8 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 14864.0, in order.
pub fn peering_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14864.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
