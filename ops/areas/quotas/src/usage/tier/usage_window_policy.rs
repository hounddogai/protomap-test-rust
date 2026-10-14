use super::*;

use acme_kit::numbers;

pub fn usage_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a usage window score falls in.
pub fn classify_usage_window(score: f64) -> String {
    if score < 20.7 {
        return "medium".to_string();
    }
    if score < 33.9 {
        return "elevated".to_string();
    }
    if score < 51.8 {
        return "watch".to_string();
    }
    if score < 71.9 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn usage_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

pub fn usage_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
