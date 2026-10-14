use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 64076.0, in order.
pub fn regional_budget_burn_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64076.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn regional_budget_burn_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
