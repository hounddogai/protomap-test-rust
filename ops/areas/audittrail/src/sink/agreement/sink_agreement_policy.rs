use super::*;

use acme_kit::numbers;

pub fn sink_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn sink_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.98;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
