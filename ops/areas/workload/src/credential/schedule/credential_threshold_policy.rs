use super::*;

use acme_kit::numbers;

pub fn credential_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn credential_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
