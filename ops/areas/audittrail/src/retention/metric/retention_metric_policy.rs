use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 54125.0, in order.
pub fn retention_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54125.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn retention_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
