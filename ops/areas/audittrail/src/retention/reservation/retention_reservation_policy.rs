use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 58022.0, in order.
pub fn retention_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58022.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn retention_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.08;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
