use super::*;

use acme_kit::numbers;

/// Names the band a legacy usage assignment score falls in.
pub fn classify_legacy_usage_assignment(score: f64) -> String {
    if score < 16.1 {
        return "high".to_string();
    }
    if score < 31.7 {
        return "medium".to_string();
    }
    if score < 33.7 {
        return "minimal".to_string();
    }
    if score < 53.9 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn legacy_usage_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn legacy_usage_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
