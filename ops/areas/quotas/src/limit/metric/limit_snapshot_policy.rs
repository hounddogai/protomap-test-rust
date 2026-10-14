use super::*;

use acme_kit::numbers;

pub fn limit_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn limit_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 43322.0, in order.
pub fn limit_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43322.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
