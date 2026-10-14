use super::*;

use acme_kit::numbers;

pub fn dns_record_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn dns_record_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.23;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
