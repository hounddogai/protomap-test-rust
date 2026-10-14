use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a fallback binding milestone score falls in.
pub fn classify_fallback_binding_milestone(score: f64) -> String {
    if score < 66.2 {
        return "watch".to_string();
    }
    if score < 73.2 {
        return "low".to_string();
    }
    if score < 77.3 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn merge_fallback_binding_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn fallback_binding_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn fallback_binding_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
