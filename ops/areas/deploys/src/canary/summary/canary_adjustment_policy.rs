use super::*;

use acme_kit::numbers;

pub fn canary_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn canary_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn canary_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a canary adjustment score falls in.
pub fn classify_canary_adjustment(score: f64) -> String {
    if score < 16.9 {
        return "watch".to_string();
    }
    if score < 17.1 {
        return "critical".to_string();
    }
    if score < 33.2 {
        return "elevated".to_string();
    }
    if score < 74.6 {
        return "low".to_string();
    }
    "medium".to_string()
}
