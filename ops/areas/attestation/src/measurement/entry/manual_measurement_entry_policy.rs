use super::*;

use acme_kit::numbers;

pub fn manual_measurement_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 68439.0, in order.
pub fn manual_measurement_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 68439.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
