use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 97309.0, in order.
pub fn suspension_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 97309.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn suspension_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
