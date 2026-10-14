use super::*;

use acme_kit::numbers;

pub fn nonce_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn nonce_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 44558.0, in order.
pub fn nonce_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44558.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
