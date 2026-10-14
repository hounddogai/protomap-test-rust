use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 50823.0, in order.
pub fn global_identity_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50823.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_identity_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn global_identity_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
