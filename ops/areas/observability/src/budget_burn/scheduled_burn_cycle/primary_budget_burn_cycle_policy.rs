use super::*;

use acme_kit::numbers;

pub fn primary_budget_burn_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 61240.0, in order.
pub fn primary_budget_burn_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 61240.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
