use super::*;

use acme_kit::numbers;

pub fn fallback_redaction_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.4;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 50931.0, in order.
pub fn fallback_redaction_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50931.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn fallback_redaction_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
