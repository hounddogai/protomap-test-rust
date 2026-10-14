use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 71081.0, in order.
pub fn plan_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 71081.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn plan_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
