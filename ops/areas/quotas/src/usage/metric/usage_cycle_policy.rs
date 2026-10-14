use super::*;

use acme_kit::numbers;

pub fn usage_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a usage cycle score falls in.
pub fn classify_usage_cycle(score: f64) -> String {
    if score < 10.7 {
        return "high".to_string();
    }
    if score < 83.0 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn usage_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
