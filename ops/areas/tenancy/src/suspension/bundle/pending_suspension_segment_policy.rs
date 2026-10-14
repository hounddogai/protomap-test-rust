use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a pending suspension segment score falls in.
pub fn classify_pending_suspension_segment(score: f64) -> String {
    if score < 7.7 {
        return "low".to_string();
    }
    if score < 39.4 {
        return "critical".to_string();
    }
    if score < 50.1 {
        return "ok".to_string();
    }
    if score < 77.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn pending_suspension_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_pending_suspension_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn pending_suspension_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
