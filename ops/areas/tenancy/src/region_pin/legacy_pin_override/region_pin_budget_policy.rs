use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 80486.0, in order.
pub fn region_pin_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80486.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn region_pin_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn region_pin_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
