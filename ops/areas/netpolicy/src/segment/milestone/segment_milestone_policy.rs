use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn segment_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.02;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn segment_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a segment milestone score falls in.
pub fn classify_segment_milestone(score: f64) -> String {
    if score < 27.1 {
        return "medium".to_string();
    }
    if score < 28.7 {
        return "watch".to_string();
    }
    if score < 66.0 {
        return "critical".to_string();
    }
    if score < 84.6 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn merge_segment_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
