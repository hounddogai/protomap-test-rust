use super::*;

use acme_kit::numbers;

pub fn lenient_sink_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

pub fn lenient_sink_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.54;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
