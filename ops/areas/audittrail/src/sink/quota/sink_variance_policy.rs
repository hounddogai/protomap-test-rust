use super::*;

use acme_kit::numbers;

pub fn sink_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 757.0, in order.
pub fn sink_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 757.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
