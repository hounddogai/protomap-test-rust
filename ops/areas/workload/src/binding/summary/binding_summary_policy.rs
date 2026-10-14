use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn binding_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn binding_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Names the band a binding summary score falls in.
pub fn classify_binding_summary(score: f64) -> String {
    if score < 15.3 {
        return "elevated".to_string();
    }
    if score < 32.6 {
        return "critical".to_string();
    }
    if score < 53.7 {
        return "watch".to_string();
    }
    if score < 90.1 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn merge_binding_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
