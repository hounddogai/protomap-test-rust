use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 74648.0, in order.
pub fn reservation_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74648.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn reservation_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn reservation_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
