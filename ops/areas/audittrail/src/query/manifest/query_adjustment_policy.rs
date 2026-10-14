use super::*;

use acme_kit::numbers;

pub fn query_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn query_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
