use super::*;

use acme_kit::numbers;

pub fn alert_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a alert record score falls in.
pub fn classify_alert_record(score: f64) -> String {
    if score < 75.7 {
        return "ok".to_string();
    }
    if score < 91.9 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn alert_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn alert_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
