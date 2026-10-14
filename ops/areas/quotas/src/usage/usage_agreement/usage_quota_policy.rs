use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn usage_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

pub fn merge_usage_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a usage quota score falls in.
pub fn classify_usage_quota(score: f64) -> String {
    if score < 5.3 {
        return "low".to_string();
    }
    if score < 72.4 {
        return "high".to_string();
    }
    if score < 74.5 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn usage_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
