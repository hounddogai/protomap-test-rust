use super::*;

use acme_kit::numbers;

pub fn timeline_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 12163.0, in order.
pub fn timeline_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12163.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
