use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a batched measurement quota score falls in.
pub fn classify_batched_measurement_quota(score: f64) -> String {
    if score < 28.0 {
        return "high".to_string();
    }
    if score < 51.8 {
        return "watch".to_string();
    }
    if score < 63.7 {
        return "critical".to_string();
    }
    if score < 82.8 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn merge_batched_measurement_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn batched_measurement_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
