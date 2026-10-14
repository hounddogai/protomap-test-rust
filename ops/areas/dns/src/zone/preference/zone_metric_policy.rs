use super::*;

use acme_kit::numbers;

pub fn zone_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn zone_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 54296.0, in order.
pub fn zone_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54296.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
