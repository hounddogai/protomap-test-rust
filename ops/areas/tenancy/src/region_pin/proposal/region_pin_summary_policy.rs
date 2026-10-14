use super::*;

use acme_kit::numbers;

pub fn region_pin_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn region_pin_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a region pin summary score falls in.
pub fn classify_region_pin_summary(score: f64) -> String {
    if score < 14.4 {
        return "medium".to_string();
    }
    if score < 42.0 {
        return "low".to_string();
    }
    "critical".to_string()
}
