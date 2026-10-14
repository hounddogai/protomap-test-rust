use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 48758.0, in order.
pub fn pending_zone_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48758.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_zone_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.08;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn pending_zone_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn pending_zone_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
