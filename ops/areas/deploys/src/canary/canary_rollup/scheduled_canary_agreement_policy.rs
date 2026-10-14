use super::*;

use acme_kit::numbers;

pub fn scheduled_canary_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn scheduled_canary_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
