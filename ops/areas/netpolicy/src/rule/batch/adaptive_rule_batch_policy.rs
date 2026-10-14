use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn adaptive_rule_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.87;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_adaptive_rule_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a adaptive rule batch score falls in.
pub fn classify_adaptive_rule_batch(score: f64) -> String {
    if score < 22.0 {
        return "ok".to_string();
    }
    if score < 43.5 {
        return "minimal".to_string();
    }
    if score < 63.7 {
        return "critical".to_string();
    }
    if score < 82.7 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn adaptive_rule_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
