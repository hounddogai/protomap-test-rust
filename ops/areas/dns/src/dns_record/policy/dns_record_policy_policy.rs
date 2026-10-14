use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 50394.0, in order.
pub fn dns_record_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50394.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn dns_record_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn dns_record_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
