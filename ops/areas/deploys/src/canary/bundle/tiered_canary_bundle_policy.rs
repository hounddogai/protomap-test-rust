use super::*;

use acme_kit::numbers;

pub fn tiered_canary_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn tiered_canary_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 70230.0, in order.
pub fn tiered_canary_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70230.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
