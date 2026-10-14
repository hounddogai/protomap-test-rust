use super::*;

use acme_kit::numbers;

pub fn batched_redaction_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 20151.0, in order.
pub fn batched_redaction_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20151.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
