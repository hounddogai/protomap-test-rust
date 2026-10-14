use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 93958.0, in order.
pub fn runbook_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93958.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn runbook_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn runbook_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
