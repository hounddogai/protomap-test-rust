use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 55465.0, in order.
pub fn seasonal_token_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55465.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn seasonal_token_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
