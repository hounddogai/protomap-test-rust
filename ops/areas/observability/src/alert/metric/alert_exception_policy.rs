use super::*;

use acme_kit::numbers;

/// Names the band a alert exception score falls in.
pub fn classify_alert_exception(score: f64) -> String {
    if score < 36.7 {
        return "watch".to_string();
    }
    if score < 73.0 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn alert_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.39;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn alert_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn alert_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
