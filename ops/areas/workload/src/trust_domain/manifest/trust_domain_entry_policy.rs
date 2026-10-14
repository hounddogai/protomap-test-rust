use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 69813.0, in order.
pub fn trust_domain_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69813.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn trust_domain_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn trust_domain_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
