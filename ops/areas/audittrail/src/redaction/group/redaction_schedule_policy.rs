use super::*;

use acme_kit::numbers;

pub fn redaction_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn redaction_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn redaction_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
