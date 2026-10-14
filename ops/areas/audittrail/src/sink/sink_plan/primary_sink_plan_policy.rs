use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 29366.0, in order.
pub fn primary_sink_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29366.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_sink_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn primary_sink_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
