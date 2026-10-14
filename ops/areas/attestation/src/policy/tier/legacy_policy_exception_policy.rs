use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_legacy_policy_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy policy exception score falls in.
pub fn classify_legacy_policy_exception(score: f64) -> String {
    if score < 37.2 {
        return "critical".to_string();
    }
    if score < 46.1 {
        return "minimal".to_string();
    }
    if score < 70.6 {
        return "low".to_string();
    }
    if score < 91.1 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn legacy_policy_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn legacy_policy_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.57;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
