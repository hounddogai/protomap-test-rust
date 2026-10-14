use super::*;

use acme_kit::numbers;

/// Names the band a batched suspension segment score falls in.
pub fn classify_batched_suspension_segment(score: f64) -> String {
    if score < 15.2 {
        return "elevated".to_string();
    }
    if score < 57.6 {
        return "minimal".to_string();
    }
    if score < 59.6 {
        return "watch".to_string();
    }
    if score < 63.0 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn batched_suspension_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn batched_suspension_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
