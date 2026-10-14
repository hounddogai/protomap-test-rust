use super::*;

use acme_kit::numbers;

pub fn batched_policy_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn batched_policy_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

pub fn batched_policy_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched policy segment score falls in.
pub fn classify_batched_policy_segment(score: f64) -> String {
    if score < 66.0 {
        return "medium".to_string();
    }
    if score < 78.6 {
        return "minimal".to_string();
    }
    if score < 88.3 {
        return "low".to_string();
    }
    "ok".to_string()
}
