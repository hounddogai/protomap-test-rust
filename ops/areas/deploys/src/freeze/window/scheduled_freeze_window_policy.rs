use super::*;

use acme_kit::numbers;

/// Names the band a scheduled freeze window score falls in.
pub fn classify_scheduled_freeze_window(score: f64) -> String {
    if score < 29.3 {
        return "minimal".to_string();
    }
    if score < 69.8 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn scheduled_freeze_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn scheduled_freeze_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn scheduled_freeze_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
